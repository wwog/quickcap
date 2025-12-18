const { spawn } = require('child_process')


//运行 RUST_LOG=info cargo run

const c = spawn('cargo', ['run'], {
    env: { ...process.env, RUST_LOG: 'info' }
})

c.stdout.addListener('data', (data) => {
    console.log(data.toString())
})
