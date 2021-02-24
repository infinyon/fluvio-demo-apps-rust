// Apache 2.0 License:
//
// Copyright (c) 2020, InfinyOn Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

window.onload = () => {
    // var webSocket = null;
    var sessionId = "";

    // Load reconnecting socket to DOM
    // loadScript("scripts/reconnecting-socket.js");

    // Create and attach Bot Assistant HTML elements
    function loadAssistant() {
        // Add assistant button
        var note = createElement("img", { "src": `img/assistant/note.svg` }),
            aButton = createElement("button", {}, note);

        // Append assistant dialog
        var status = createElement("div", { "id": "bot-status", "class": "status off" }),
            overlay = createElement("div", { "class": "overlay" }, status),
            bot = createElement("img", { "src": `img/assistant/bot.svg`, "class": "bot" }),
            title = createElement("span", {}, "Bot Assistant"),
            aDialogClose = createElement("img", { "src": `img/assistant/close.svg`, "class": "close" }),
            aDialogReset = createElement("img", { "src": `img/assistant/redo.svg` }),
            header = createElement("div", { "class": "header" }, [bot, overlay, title, aDialogClose, aDialogReset]),
            msgBody = createElement("div", { "class": "msg-body" }),
            innerBody = createElement("div", { "class": "inner-body" }, msgBody),
            body = createElement("div", { "class": "body-wrapper" }, innerBody),
            userMsg = createElement("div", {
                "id": "user-msg",
                "class": "textareaElement",
                "placeholder": "Choose an option",
                "contenteditable": "false"
            }),
            footer = createElement("div", { "class": "footer" }, userMsg),
            aDialog = createElement("div", { "class": "chat" }, [header, body, footer]);

        // Attach event listeners
        aButton.addEventListener('click', onOpenDialog, false);
        aDialogClose.addEventListener('click', onCloseDialog, false);
        aDialogReset.addEventListener('click', onResetSession, false);

        // Add to document
        document.querySelector(".assistant").appendChild(aButton);
        document.querySelector(".assistant").appendChild(aDialog);
    }

    // On open assistant dialog callback
    function onOpenDialog() {
        document.querySelector(".assistant button").style.display = "none";
        document.querySelector(".assistant .chat").style.display = "block";
        // openWSConnection();
    }

    // On close assistant dialog callback
    function onCloseDialog() {
        document.querySelector(".assistant .chat").style.display = "none";
        document.querySelector(".assistant button").style.display = "block";
    }

    // Clear the cookie and restart connection to create a new session.
    function onResetSession() {
        document.cookie = "Fluvio-Bot-Assistant=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";

        closeWsConnection();
        clearMessages();
        // openWSConnection();
    }

    // Open WebSocket connection
    // function openWSConnection() {
    //     try {
    //         if (webSocket != null) {
    //             return; // already connected
    //         }

    //         logOutput("Connecting to: ws://localhost:9998/");
    //         webSocket = new ReconnectingWebSocket("ws://localhost:9998/");

    //         webSocket.onopen = function (openEvent) {
    //             clearMessages();
    //             document.getElementById("bot-status").setAttribute("class", "status on");
    //             logOutput("Connected!");
    //         };

    //         webSocket.onclose = function (closeEvent) {
    //             document.getElementById("bot-status").setAttribute("class", "status off");
    //             logOutput("Disconnected!");
    //         };

    //         webSocket.onerror = function (errorEvent) {
    //             logOutput(`Error: ${JSON.stringify(errorEvent)}`);
    //         };

    //         webSocket.onmessage = function (messageEvent) {
    //             var serverMsg = messageEvent.data;
    //             logOutput(`<== ${serverMsg}`);
    //             onMessageFromServer(serverMsg);
    //         };

    //     } catch (exception) {
    //         logOutput(`error: ${JSON.stringify(exception)}`);
    //     }
    // }

    // Close WS Connection
    // function closeWsConnection() {
    //     if (webSocket.open) {
    //         webSocket.close();
    //         webSocket = null;
    //     }
    // }

    // On messages received from Websocket
    function onMessageFromServer(value) {
        const message = JSON.parse(value);
        switch (message.kind) {
            case "BotText":
                showBotText(message.content);
                break;
            case "UserText":
                showUserText(message.content);
                break;
            case "ChoiceRequest":
                showBotText(message.question);
                showChoiceButtons(message.groupId, message.choices);
                break;
            case "ChoiceResponse":
                choicesToButton(message.groupId, message.content);
                break;
            case "StartChatSession":
                sessionId = message.sessionId;
                enableChatEditor(message.chatPrompt, message.chatText);
                break;
            case "EndChatSession":
                disableChatEditor();
                break;
        };
    }

    // Send a message on WebSocket
    function sendWsMessage(message) {
        if (webSocket.readyState != WebSocket.OPEN) {
            logOutput("WebSocket is not connected: " + webSocket.readyState);
            return;
        }

        const msgObj = JSON.stringify(message)
        logOutput(`==> ${msgObj}`);

        webSocket.send(msgObj);
    }

    // Show text from bot assistant
    function showBotText(content) {
        if (content.length > 0) {
            removeDuplicateAvatar("bot");

            var img = createElement("img", { "src": `img/assistant/bot.svg` }),
                avatar = createElement("div", { "class": "avatar", "id": "bot" }, img),
                msg = createElement("div", { "class": "msg" }, content),
                msgLeft = createElement("div", { "class": "msg-left" }, [msg, avatar]);

            document.querySelector(".msg-body").appendChild(msgLeft);
            scrollToBottom(".inner-body");
        }
    }

    // Show text from user interactive session
    function showUserText(content) {
        if (content.length > 0) {
            var msg = createElement("div", { "class": "msg" }, content),
                msgLeft = createElement("div", { "class": "msg-right" }, msg);

            document.querySelector(".msg-body").appendChild(msgLeft);
            scrollToBottom(".inner-body");
        }
    }

    // Show choices
    function showChoiceButtons(groupId, choices) {
        if (choices.length > 0) {
            var buttons = [];

            choices.forEach(choice => {
                var button = createElement("div", { "class": "button" }, choice.content);
                button.addEventListener('click', function () {
                    pickChoice(groupId, choice.itemId, choice.content);
                }, false);

                buttons.push(createElement("div", { "class": "btn" }, button));
            });

            var msgLeft = createElement("div", { "class": "msg-left", "id": groupId }, buttons);

            document.querySelector(".msg-body").appendChild(msgLeft);
            scrollToBottom(".inner-body");
        }
    }

    // Callback invoked on user selection
    function pickChoice(groupId, itemId, content) {
        choicesToButton(groupId, content);

        sendWsMessage({
            kind: "ChoiceResponse",
            groupId: groupId,
            itemId: itemId,
            content: content,
        });
    }

    // Swap choices with a button representing the selection
    function choicesToButton(groupId, content) {
        document.getElementById(groupId).remove();

        var button = createElement("div", { "class": "button selected" }, content),
            btn = createElement("div", { "class": "btn" }, button),
            msgRight = createElement("div", { "class": "msg-right" }, btn);

        document.querySelector(".msg-body").appendChild(msgRight);
        scrollToBottom(".inner-body");
    }

    // On multiple bot messages, ensure avatar is only displayed on last entry
    function removeDuplicateAvatar(id) {
        var messages = document.querySelector('.msg-body').children;
        if (messages.length > 0) {
            var lastMessage = messages[messages.length - 1];
            if (lastMessage.getAttribute("class") === 'msg-left') {
                if (lastMessage.lastChild.id == id) {
                    lastMessage.removeChild(lastMessage.lastChild);
                }
            }
        }
    }

    // Enable interactive chat
    function enableChatEditor(chatPrompt, chatText) {
        if (chatText) {
            showBotText(chatText);
        }

        var chatBox = document.getElementById("user-msg");
        chatBox.setAttribute("contenteditable", true);
        chatBox.setAttribute("placeholder", chatPrompt || "Type question here ...");

        chatBox.addEventListener("keydown", onEditorKeys, false);
    }

    // Disable interactive chat
    function disableChatEditor() {
        var chatBox = document.getElementById("user-msg");
        chatBox.addEventListener("keydown", {}, false);

        chatBox.setAttribute("contenteditable", false);
        chatBox.setAttribute("placeholder", "Choose an option");
    }

    // Scroll to last messages
    function scrollToBottom(tag) {
        var div = document.querySelector(tag);
        div.scrollTop = div.scrollHeight - div.clientHeight;
    }

    // Clear messages in both editors
    function clearMessages() {
        var parent = document.querySelector('.msg-body');
        while (parent.firstChild) {
            parent.removeChild(parent.firstChild);
        }

        var debugOutput = document.getElementById("debugOutput");
        if (debugOutput) {
            debugOutput.value = "";
        }
    }

    // Capture editor keys
    function onEditorKeys(e) {
        var chatBox = document.getElementById("user-msg");

        if (e.code == 'Enter' && chatBox.textContent.length > 0) {
            e.preventDefault();

            const content = chatBox.textContent;
            sendWsMessage({
                kind: "UserText",
                sessionId: sessionId,
                content: content,
            });
            showUserText(content);

            chatBox.innerHTML = '';
        }
    }

    //  Load external javascript file to DOM
    function loadScript(fileName) {
        var js_script = document.createElement('script');
        js_script.type = "text/javascript";
        js_script.src = fileName;
        js_script.async = false;
        document.getElementsByTagName('head')[0].appendChild(js_script);
    }

    // Log output in the "debugOutput" textarea (if available) and the console
    function logOutput(value) {
        var debugOutput = document.getElementById("debugOutput");
        if (debugOutput) {
            debugOutput.value += value + "\n\n";
            debugOutput.scrollTop = debugOutput.scrollHeight;
        }
        console.log(value);
    }

    // Create element utility function
    function createElement(element, attribute, inner) {
        if (typeof (element) === "undefined") { return false; }
        if (typeof (inner) === "undefined") { inner = ""; }

        var el = document.createElement(element);
        if (typeof (attribute) === 'object') {
            for (var key in attribute) {
                el.setAttribute(key, attribute[key]);
            }
        }
        if (!Array.isArray(inner)) {
            inner = [inner];
        }
        for (var k = 0; k < inner.length; k++) {
            if (inner[k].tagName) {
                el.appendChild(inner[k]);
            } else {
                el.innerHTML = inner[k];
            }
        }
        return el;
    }

    // Call main function
    loadAssistant();
};