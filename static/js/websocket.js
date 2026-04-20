class WebSocketClient {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.messageHandlers = new Map();
        this.isConnected = false;
    }

    connect(token, deviceId) {
        const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws?token=${encodeURIComponent(token)}&device_id=${encodeURIComponent(deviceId)}`;
        
        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.isConnected = true;
            this.reconnectAttempts = 0;
            this.emit('connected');
        };

        this.ws.onclose = (event) => {
            console.log('WebSocket disconnected:', event.code, event.reason);
            this.isConnected = false;
            this.emit('disconnected');
            
            if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
                setTimeout(() => {
                    this.reconnectAttempts++;
                    console.log(`Reconnecting... Attempt ${this.reconnectAttempts}`);
                    this.connect(token, deviceId);
                }, this.reconnectDelay * this.reconnectAttempts);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.emit('error', error);
        };

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                this.handleMessage(data);
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };
    }

    handleMessage(data) {
        const { type, payload } = data;
        
        switch (type) {
            case 'connected':
                console.log('WebSocket connected:', payload);
                this.emit('connected', payload);
                break;
            case 'message':
                this.emit('message', payload);
                break;
            case 'message_sent':
                this.emit('message_sent', payload);
                break;
            case 'typing':
                this.emit('typing', payload);
                break;
            case 'user_online':
                this.emit('user_online', payload);
                break;
            case 'user_offline':
                this.emit('user_offline', payload);
                break;
            case 'message_read':
                this.emit('message_read', payload);
                break;
            case 'group_member_joined':
                this.emit('group_member_joined', payload);
                break;
            case 'group_member_left':
                this.emit('group_member_left', payload);
                break;
            case 'error':
                console.error('Server error:', payload);
                this.emit('error', payload);
                break;
            default:
                console.log('Unknown message type:', type, data);
        }
    }

    send(type, payload) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ type, payload }));
        } else {
            console.warn('WebSocket is not connected');
        }
    }

    sendMessage(conversationId, content, tempId) {
        this.send('send_message', {
            conversation_id: conversationId,
            content,
            temp_id: tempId,
        });
    }

    sendTyping(conversationId) {
        this.send('typing', { conversation_id: conversationId });
    }

    markAsRead(conversationId, messageId) {
        this.send('mark_read', {
            conversation_id: conversationId,
            message_id: messageId,
        });
    }

    on(event, callback) {
        if (!this.messageHandlers.has(event)) {
            this.messageHandlers.set(event, []);
        }
        this.messageHandlers.get(event).push(callback);
    }

    off(event, callback) {
        if (this.messageHandlers.has(event)) {
            const handlers = this.messageHandlers.get(event);
            const index = handlers.indexOf(callback);
            if (index > -1) {
                handlers.splice(index, 1);
            }
        }
    }

    emit(event, data) {
        if (this.messageHandlers.has(event)) {
            this.messageHandlers.get(event).forEach(callback => callback(data));
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close(1000, 'User logout');
            this.ws = null;
        }
        this.isConnected = false;
    }
}

const wsClient = new WebSocketClient();
