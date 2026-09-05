#!/usr/bin/env python3
"""Exercise a local candidate through public app-server APIs with an isolated mock model."""
import argparse
from collections import deque
import gzip
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
from queue import Queue
import subprocess
import tempfile
import threading


class MockModel(BaseHTTPRequestHandler):
    requests = []

    def log_message(self, *_args):
        pass

    def do_POST(self):
        body = self.rfile.read(int(self.headers['Content-Length']))
        encoding = self.headers.get('Content-Encoding')
        if encoding == 'zstd':
            body = subprocess.run(['zstd', '-dc'], input=body, capture_output=True, check=True).stdout
        elif encoding == 'gzip':
            body = gzip.decompress(body)
        elif encoding is not None:
            raise ValueError(f'Unsupported request encoding: {encoding}')
        self.requests.append(json.loads(body))
        response_id = f'smoke-{len(self.requests)}'
        events = [
            {'type': 'response.created', 'response': {'id': response_id}},
            {'type': 'response.output_item.done', 'item': {
                'type': 'message', 'id': f'message-{response_id}', 'role': 'assistant',
                'content': [{'type': 'output_text', 'text': 'Candidate smoke completed.'}],
            }},
            {'type': 'response.completed', 'response': {'id': response_id}},
        ]
        if len(self.requests) == 1:
            events[1]['item'] = {
                'type': 'custom_tool_call', 'id': 'smoke-code-mode',
                'call_id': 'smoke-exec-call', 'name': 'exec',
                'input': 'text(await tools.exec_command({cmd: "printf spine-package-smoke", login: false}));',
            }
        payload = ''.join(f'data: {json.dumps(event)}\n\n' for event in events).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


class AppServer:
    def __init__(self, binary, environment, stderr):
        self.process = subprocess.Popen(
            [str(binary), 'app-server'], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=stderr, env=environment, text=True, bufsize=1,
        )
        self.messages = Queue()
        self.pending = deque()
        self.next_id = 0
        threading.Thread(target=self.read_messages, daemon=True).start()
        initialized = self.request('initialize', {
            'clientInfo': {'name': 'spine-migration-smoke', 'version': '0.4.0'},
            'capabilities': {'experimentalApi': True},
        })
        assert '/0.153.4 ' in initialized['userAgent'], initialized
        self.send({'method': 'initialized'})

    def read_messages(self):
        for line in self.process.stdout:
            self.messages.put(json.loads(line))
        self.messages.put(None)

    def send(self, message):
        self.process.stdin.write(json.dumps(message) + '\n')
        self.process.stdin.flush()

    def request(self, method, params):
        self.next_id += 1
        self.send({'id': self.next_id, 'method': method, 'params': params})
        while True:
            message = self.messages.get(timeout=30)
            assert message is not None, 'app-server closed before response'
            if message.get('id') == self.next_id:
                assert 'error' not in message, message
                return message['result']
            self.pending.append(message)

    def turn(self, thread_id, text):
        self.request('turn/start', {'threadId': thread_id, 'input': [{'type': 'text', 'text': text}]})
        while True:
            message = self.pending.popleft() if self.pending else self.messages.get(timeout=30)
            assert message is not None, 'app-server closed before completion'
            if message.get('method') == 'turn/completed':
                assert message['params']['threadId'] == thread_id
                assert message['params']['turn']['status'] == 'completed', message
                return

    def close(self):
        self.process.stdin.close()
        assert self.process.wait(timeout=30) == 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--log-dir', type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    args.log_dir.mkdir(parents=True, exist_ok=True)
    version = subprocess.check_output([str(binary), '--version'], text=True).strip()
    assert version.endswith('0.153.4'), version
    model = ThreadingHTTPServer(('127.0.0.1', 0), MockModel)
    threading.Thread(target=model.serve_forever, daemon=True).start()
    with tempfile.TemporaryDirectory(prefix='spine-candidate-') as temporary:
        root = Path(temporary)
        home = root / 'home'
        workspace = root / 'workspace'
        home.mkdir()
        workspace.mkdir()
        (home / 'config.toml').write_text(f'''model = "gpt-5.4"
model_provider = "migration_mock"
approval_policy = "never"
[model_providers.migration_mock]
name = "Migration mock"
base_url = "http://127.0.0.1:{model.server_port}/v1"
wire_api = "responses"
env_key = "MIGRATION_MOCK_KEY"
supports_websockets = false
[features]
code_mode = true
''')
        environment = dict(os.environ, CODEX_HOME=str(home), MIGRATION_MOCK_KEY='synthetic-test-key')
        environment['NO_PROXY'] = '127.0.0.1,localhost'
        environment['no_proxy'] = environment['NO_PROXY']
        with (args.log_dir / 'app-server.stderr').open('w') as stderr:
            app = AppServer(binary, environment, stderr)
            started = app.request('thread/start', {'cwd': str(workspace), 'approvalPolicy': 'never'})
            thread_id = started['thread']['id']
            app.turn(thread_id, 'first isolated migration smoke turn')
            app.close()
            app = AppServer(binary, environment, stderr)
            resumed = app.request('thread/resume', {'threadId': thread_id, 'excludeTurns': True})
            assert resumed['thread']['id'] == thread_id
            app.turn(thread_id, 'second isolated migration smoke turn')
            app.close()
        assert len(MockModel.requests) == 3, len(MockModel.requests)
        outputs = [item for item in MockModel.requests[1]['input'] if item.get('call_id') == 'smoke-exec-call' and item['type'] == 'custom_tool_call_output']
        assert len(outputs) == 1, outputs
        execution = json.loads(outputs[0]['output'][-1]['text'])
        assert (execution['exit_code'], execution['output']) == (0, 'spine-package-smoke'), execution
        assert 'spine' in json.dumps(MockModel.requests[0]['tools'])
        assert 'first isolated migration smoke turn' in json.dumps(MockModel.requests[2]['input'])
        assert 'second isolated migration smoke turn' in json.dumps(MockModel.requests[2]['input'])
        (args.log_dir / 'requests.json').write_text(json.dumps(MockModel.requests, indent=2))
    model.shutdown()
    print(json.dumps({'cliVersion': version, 'requests': 3, 'codeModeExecution': 'passed', 'resume': 'passed', 'shutdown': 'passed'}))


if __name__ == '__main__':
    main()
