use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn index() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Terma - Terminal Chat</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            background: #ffffff;
            color: #000000;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 2rem;
        }

        .container {
            max-width: 42rem;
            width: 100%;
        }

        h1 {
            font-size: 3rem;
            font-weight: 400;
            letter-spacing: -0.05em;
            margin-bottom: 1rem;
        }

        p {
            font-size: 1rem;
            color: #666666;
            line-height: 1.6;
            margin-bottom: 3rem;
        }

        .card {
            border: 1px solid #e5e5e5;
            background: #fafafa;
            padding: 2.5rem;
            margin-bottom: 2rem;
        }

        button {
            background: transparent;
            border: 1px solid #000000;
            color: #000000;
            padding: 0.75rem 1.5rem;
            font-size: 0.875rem;
            font-weight: 500;
            letter-spacing: 0.05em;
            cursor: pointer;
            transition: all 0.2s;
            font-family: 'Inter', sans-serif;
            text-transform: uppercase;
        }

        button:hover:not(:disabled) {
            background: #000000;
            color: #ffffff;
        }

        button:disabled {
            opacity: 0.4;
            cursor: not-allowed;
        }

        .command-box {
            display: none;
            margin-top: 2rem;
        }

        .command-box.active {
            display: block;
        }

        .label {
            font-size: 0.75rem;
            font-weight: 500;
            letter-spacing: 0.1em;
            text-transform: uppercase;
            color: #666666;
            margin-bottom: 0.5rem;
        }

        .command {
            background: #ffffff;
            border: 1px solid #e5e5e5;
            padding: 1rem;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.875rem;
            word-break: break-all;
            margin-bottom: 0.75rem;
        }

        .copy-btn {
            font-size: 0.75rem;
            padding: 0.5rem 1rem;
        }

        .divider {
            height: 1px;
            background: #e5e5e5;
            margin: 2rem 0;
        }

        .footer {
            text-align: center;
            color: #999999;
            font-size: 0.875rem;
            margin-top: 3rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Terma</h1>
        <p>Real-time terminal chat. Create a room, share the link, and start chatting instantly from your terminal.</p>

        <div class="card">
            <button id="createBtn" onclick="createRoom()">Create New Room</button>

            <div id="commandBox" class="command-box">
                <div class="label">Run this command in your terminal</div>
                <div id="command" class="command"></div>
                <button id="copyCommandBtn" class="copy-btn" onclick="copyCommand()">Copy Command</button>

                <div style="margin-top: 2rem;">
                    <div class="label">Share this link with others</div>
                    <div id="shareLink" class="command"></div>
                    <button id="copyLinkBtn" class="copy-btn" onclick="copyLink()">Copy Link</button>
                </div>

                <div class="divider"></div>
                <button id="createAnotherBtn" style="display: none;" onclick="createRoom()">Create Another Room</button>
            </div>
        </div>

        <div class="footer">
            Built with Rust, Axum, and Ratatui
        </div>
    </div>

    <script>
        async function createRoom() {
            const btn = document.getElementById('createBtn');
            const anotherBtn = document.getElementById('createAnotherBtn');
            const activeBtn = btn.style.display === 'none' ? anotherBtn : btn;

            activeBtn.disabled = true;
            activeBtn.textContent = 'Creating...';

            try {
                const response = await fetch('/api/rooms', {
                    method: 'POST',
                });

                if (!response.ok) throw new Error('Failed to create room');

                const data = await response.json();
                displayRoom(data.room_id, data.install_command);
            } catch (error) {
                alert('Error creating room: ' + error.message);
                activeBtn.textContent = btn.style.display === 'none' ? 'Create Another Room' : 'Create New Room';
            } finally {
                activeBtn.disabled = false;
            }
        }

        function displayRoom(roomId, installCommand) {
            document.getElementById('command').textContent = installCommand;
            document.getElementById('shareLink').textContent = window.location.origin + '/#' + roomId;
            document.getElementById('commandBox').classList.add('active');
            document.getElementById('createBtn').style.display = 'none';
            document.getElementById('createAnotherBtn').style.display = 'block';
            window.location.hash = roomId;
        }

        function copyCommand() {
            const btn = document.getElementById('copyCommandBtn');
            const command = document.getElementById('command').textContent;
            navigator.clipboard.writeText(command);

            const originalText = btn.textContent;
            btn.textContent = 'Copied!';
            setTimeout(() => {
                btn.textContent = originalText;
            }, 2000);
        }

        function copyLink() {
            const btn = document.getElementById('copyLinkBtn');
            const link = document.getElementById('shareLink').textContent;
            navigator.clipboard.writeText(link);

            const originalText = btn.textContent;
            btn.textContent = 'Copied!';
            setTimeout(() => {
                btn.textContent = originalText;
            }, 2000);
        }

        // Handle hash on page load
        window.addEventListener('DOMContentLoaded', () => {
            const hash = window.location.hash.slice(1);
            if (hash) {
                const host = window.location.host;
                const protocol = window.location.protocol;
                const installCommand = `sh -c "$(curl -fsSL ${protocol}//${host}/join/${hash})"`;
                displayRoom(hash, installCommand);
            }
        });
    </script>
</body>
</html>"#;

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], html).into_response()
}
