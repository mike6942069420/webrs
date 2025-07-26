console.log("If you see this message, it means the JavaScript is working correctly on www.bigmike.ch!");

const protocol = window.location.protocol === "https:" ? "wss://" : "ws://";
const host = window.location.host;
const socketUrl = protocol + host + "/ws";
const socket = new WebSocket(socketUrl);

const messageBox = document.getElementById("messageBox");
const form = document.querySelector("form.form-container");
const input = form.querySelector("input[name='message']");

let clientID = 0;

function exit() {
    if (socket.readyState === WebSocket.OPEN) {
        socket.close();
    }
    console.log("Exiting the webSocket connection.");
}

function start() {
    // reload the page to reset the client ID
    window.location.reload();
}

function getClientInfo() {
    return {
        id: clientID,
        type: "info",
        content: {
            width: window.screen.width,
            height: window.screen.height,
            availWidth: window.screen.availWidth,
            availHeight: window.screen.availHeight,
            pixelDepth: window.screen.pixelDepth,
            colorDepth: window.screen.colorDepth,
            devicePixelRatio: window.devicePixelRatio,

            userAgent: navigator.userAgent,
            platform: navigator.platform,
            language: navigator.language,
            languages: navigator.languages,
            hardwareConcurrency: navigator.hardwareConcurrency || 'Unavailable',
            deviceMemory: navigator.deviceMemory || 'Unavailable',

            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            offset: new Date().getTimezoneOffset(),

            online: navigator.onLine,
            cookieEnabled: navigator.cookieEnabled,
            doNotTrack: navigator.doNotTrack,

            pageUrl: window.location.href,
            referrer: document.referrer,
            pageTitle: document.title,

            innerWidth: window.innerWidth,
            innerHeight: window.innerHeight,
            clientWidth: document.documentElement.clientWidth,
            clientHeight: document.documentElement.clientHeight,

            ontouchstart: 'ontouchstart' in window,
            maxTouchPoints: navigator.maxTouchPoints || 0,

            localeString: new Date().toLocaleString(),
            fullDate: new Date().toString(),

            webAssembly: typeof WebAssembly !== 'undefined',
            sharedArrayBuffer: typeof SharedArrayBuffer !== 'undefined',
            offscreenCanvas: typeof OffscreenCanvas !== 'undefined'
        }
    };
}

// Append message helper
function appendMessage(msg, color = "white", type = "other") {
    const div = document.createElement("div");
    div.className = "message message-" + type;
    div.textContent = msg;
    div.style.color = color;
    messageBox.appendChild(div);
    messageBox.scrollTop = messageBox.scrollHeight;
}

window.addEventListener("DOMContentLoaded", () => {
    messageBox.scrollTop = messageBox.scrollHeight;
});

socket.addEventListener("open", () => {
    console.log("WebSocket is open now.");
});

socket.addEventListener("message", async (event) => {
    try {
        const message = await JSON.parse(event.data);
        
        //console.log("Message received:", message);
        
        if (clientID === 0 && message.type !== "id") {
            if (message.type === "error") {
                showNotification(message.content);
            } else {
                showNotification("Failed to receive client ID. Please reload the page.");
                socket.close();
            }
            return;
        }

        switch (message.type) {
            case "id":
                clientID = parseInt(message.content);
                console.log("Client ID received:", clientID);
                socket.send(JSON.stringify(getClientInfo()));
                break;

            case "message":
                if (message.id === clientID) {
                    console.log("Message broadcasted to all");
                } else {
                    appendMessage(message.content, "white", "other");
                }
                break;

            case "error":
                showNotification(message.content);
                break;

            case "nbusers":
                const userCountDiv = document.getElementById("userCount");
                userCountDiv.textContent = `Connected users: ${message.content}`;
                break;

            default:
                console.warn("Unknown message type received:", message.type);
                break;
        }
    } catch (e) {
        console.error("Failed to parse message:", event.data, e);
        if (clientID === 0) {
            showNotification("Failed to receive client ID. Please reload the page.");
            socket.close();
        }
    }
});

socket.addEventListener("error", (event) => {
    console.error("WebSocket error:", event);
    const userCountDiv = document.getElementById("userCount");
    userCountDiv.textContent = `You are offline`;
});

socket.addEventListener("close", () => {
    setTimeout(() => {
        console.log("WebSocket is closed now.");
        const userCountDiv = document.getElementById("userCount");
        userCountDiv.textContent = `You are offline`;
    }, 500);
});

function showNotification(message, duration = 5000) {
    const notif = document.getElementById('notification');
    notif.textContent = message;
    notif.classList.add('show');
    setTimeout(() => {
        notif.classList.remove('show');
    }, duration);
}

form.addEventListener("submit", (e) => {
    e.preventDefault();

    if (clientID == 0) {
        showNotification("Client ID not received yet. Please wait or reload the page.");
        return;
    }

    let msg = input.value;

    if (msg.trim() === "") {
        showNotification("Message cannot be empty.");
        input.value = "";
        input.focus();
        return;
    }

    if (msg.length > 200) {
        msg = msg.slice(0, 200);
    }

    if (socket.readyState === WebSocket.OPEN) {
        if (msg.startsWith("/")) {
            const localCommand = msg.slice(1).trim().split(" ")[0].toLowerCase();

            switch (localCommand) {
                case "clear":
                    messageBox.innerHTML = "";
                    break;
                case "info":
                    appendMessage(JSON.stringify(getClientInfo()), "white", "local");
                    break;
                case "help":
                    const helpMessage = `Available commands:
- /help: Show this help message.
- /clear: Clear the message box.
- /info: Show the client information.
- /echo [message]: echo a message
`;
                    appendMessage(helpMessage, "white", "local");
                    break;
                case "echo":
                    const logMessage = msg.slice(5).trim();
                    if (logMessage) {
                        console.log("echo message:", logMessage);
                        appendMessage(logMessage, "white", "local");
                    } else {
                        showNotification("Usage: /echo [message]", 1500);
                    }
                    break;
                default:
                    showNotification(`Unknown command: ${localCommand}. Type /help for available commands.`, 1500);
                    break;
            }

            // Send local command as binary JSON
            socket.send(JSON.stringify({ id: clientID, type: "local", content: msg }));
            input.value = "";
            input.focus();
        } else {
            appendMessage(msg, "white", "self");
            socket.send(JSON.stringify({ id: clientID, type: "message", content: msg }));
            input.value = "";
            input.focus();
        }
    } else {
        showNotification("WebSocket is closed. Please reload the page or try again later.");
    }
});