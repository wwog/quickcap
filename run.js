const { spawn } = require('child_process')


//运行 RUST_LOG=info cargo run

const c = spawn('cargo', ['run'], {
    env: { ...process.env, RUST_LOG: 'info' },
    stdio: ['pipe', 'pipe', 'inherit'],
})

c.stdout.on('data', (data) => {
    const lines = data.toString().split('\n')
    for (const line of lines) {
        const trimmed = line.trim()
        if (trimmed.startsWith('{')) {
            try {
                const msg = JSON.parse(trimmed)
                console.log('JS LOG:[RPC Notification]', msg)
                if (msg.method === 'test') {
                    console.log("JS LOG: Received test notification")
                    const msg = JSON.stringify({
                        jsonrpc: '2.0',
                        method: 'recv_test',
                        params: { message: 'Hello, world!' }
                    });

                    c.stdin.write(msg + '\n');
                }
            } catch (e) {
                console.log(line)
            }
        } else if (trimmed) {
            console.log(line)
        }
    }
})
