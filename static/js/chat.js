let currentConversation = null;
let conversations = [];
let groups = [];
let messages = {};
let currentTab = 'chats';
let deviceId = null;
let pendingMessages = new Map();

document.addEventListener('DOMContentLoaded', async () => {
    const token = localStorage.getItem('access_token');
    if (!token) {
        window.location.href = '/';
        return;
    }

    const userStr = localStorage.getItem('user');
    if (userStr) {
        const user = JSON.parse(userStr);
        document.getElementById('user-display').textContent = user.username || user.display_name || 'User';
    }

    deviceId = localStorage.getItem('device_id') || generateDeviceId();
    localStorage.setItem('device_id', deviceId);

    initWebSocket();
    await loadConversations();
    await loadGroups();

    document.getElementById('sidebar-toggle').addEventListener('click', toggleSidebar);
    document.getElementById('message-input').addEventListener('input', handleTyping);

    const startChatWith = localStorage.getItem('start_chat_with');
    if (startChatWith) {
        localStorage.removeItem('start_chat_with');
        await startConversationWithFriend(startChatWith);
    }
});

async function startConversationWithFriend(friendId) {
    try {
        const response = await createConversation([friendId]);
        if (response) {
            const conv = {
                id: response.id,
                conversation_type: 'direct',
                other_user_id: friendId,
                display_name: 'Friend'
            };
            conversations.unshift(conv);
            renderConversations();
            openConversation(conv);
        }
    } catch (error) {
        console.error('Failed to start conversation:', error);
    }
}

function generateDeviceId() {
    return crypto.randomUUID();
}

function initWebSocket() {
    const token = localStorage.getItem('access_token');
    wsClient.connect(token, deviceId);

    wsClient.on('connected', () => {
        console.log('WebSocket connected');
    });

    wsClient.on('message', (data) => {
        const message = {
            id: data.id,
            conversation_id: data.conversation_id,
            sender_id: data.sender_id,
            content: data.content,
            message_type: data.message_type,
            created_at: data.created_at,
            status: 'delivered',
        };
        addMessageToUI(message);
        updateConversationLastMessage(message);
    });

    wsClient.on('message_sent', (data) => {
        const tempId = pendingMessages.get(data.seq);
        const message = {
            id: data.id,
            temp_id: tempId,
            conversation_id: data.conversation_id,
            sender_id: data.sender_id,
            content: data.content,
            message_type: data.message_type,
            created_at: data.created_at,
            status: 'sent',
        };
        addMessageToUI(message);
        updateConversationLastMessage(message);
    });

    wsClient.on('typing', (data) => {
        if (currentConversation && currentConversation.id === data.conversation_id) {
            showTypingIndicator(data.user_name || 'Someone');
        }
    });

    wsClient.on('user_online', (data) => {
        updateOnlineStatus(data.user_id, true);
    });

    wsClient.on('user_offline', (data) => {
        updateOnlineStatus(data.user_id, false);
    });

    wsClient.on('disconnected', () => {
        console.log('WebSocket disconnected');
    });
}

async function loadConversations() {
    try {
        const response = await getConversations();
        conversations = response.conversations || [];
        renderConversations();
    } catch (error) {
        console.error('Failed to load conversations:', error);
    }
}

async function loadGroups() {
    try {
        const response = await getGroups();
        groups = response.groups || [];
        renderConversations();
    } catch (error) {
        console.error('Failed to load groups:', error);
    }
}

function renderConversations() {
    const list = document.getElementById('conversation-list');
    list.innerHTML = '';

    const items = currentTab === 'chats' ? conversations : groups;

    if (items.length === 0) {
        list.innerHTML = `
            <div class="p-4 text-center text-gray-500">
                <p>No ${currentTab} yet</p>
                <p class="text-sm mt-1">Start a new conversation!</p>
            </div>
        `;
        return;
    }

    items.forEach(item => {
        const div = document.createElement('div');
        div.className = `p-4 hover:bg-gray-50 cursor-pointer border-b border-gray-100 ${currentConversation?.id === item.id ? 'bg-purple-50' : ''}`;
        div.onclick = () => openConversation(item);

        const avatar = item.avatar || getInitials(item.name || item.display_name || 'U');
        const lastMessage = item.last_message?.content || 'No messages yet';
        const time = item.last_message?.created_at ? formatTime(item.last_message.created_at) : '';
        const unread = item.unread_count || 0;

        div.innerHTML = `
            <div class="flex items-center">
                <div class="relative">
                    <div class="w-12 h-12 rounded-full bg-gradient-to-r from-purple-600 to-indigo-600 flex items-center justify-center text-white font-semibold">
                        ${avatar}
                    </div>
                    ${item.is_online ? '<div class="online-indicator"></div>' : ''}
                </div>
                <div class="ml-3 flex-1 min-w-0">
                    <div class="flex justify-between items-center">
                        <h4 class="font-medium text-gray-800 truncate">${escapeHtml(item.name || item.display_name || 'Unknown')}</h4>
                        ${time ? `<span class="text-xs text-gray-500">${time}</span>` : ''}
                    </div>
                    <div class="flex justify-between items-center">
                        <p class="text-sm text-gray-500 truncate">${escapeHtml(lastMessage)}</p>
                        ${unread > 0 ? `<span class="bg-purple-600 text-white text-xs rounded-full px-2 py-0.5">${unread}</span>` : ''}
                    </div>
                </div>
            </div>
        `;

        list.appendChild(div);
    });
}

async function openConversation(conversation) {
    currentConversation = conversation;

    document.getElementById('no-chat-selected').classList.add('hidden');
    document.getElementById('chat-view').classList.remove('hidden');

    const title = conversation.name || conversation.display_name || 'Direct Chat';
    document.getElementById('chat-title').textContent = title;
    document.getElementById('chat-avatar').textContent = getInitials(title);

    if (conversation.conversation_type === 'direct') {
        document.getElementById('chat-status').textContent = conversation.is_online ? 'Online' : 'Offline';
        document.getElementById('chat-online-status').classList.toggle('hidden', !conversation.is_online);
    } else {
        document.getElementById('chat-status').textContent = `${conversation.member_count || 0} members`;
        document.getElementById('chat-online-status').classList.add('hidden');
        document.getElementById('create-group-btn').classList.remove('hidden');
    }

    if (window.innerWidth < 768) {
        document.getElementById('sidebar').classList.add('sidebar-hidden');
    }

    await loadMessages(conversation.id);
    renderConversations();
}

async function loadMessages(conversationId) {
    try {
        const response = await getMessages(conversationId);
        messages[conversationId] = response.messages || [];
        renderMessages();
    } catch (error) {
        console.error('Failed to load messages:', error);
    }
}

function renderMessages() {
    const list = document.getElementById('message-list');
    const msgs = messages[currentConversation?.id] || [];
    const currentUserId = JSON.parse(localStorage.getItem('user'))?.id;

    list.innerHTML = '';

    msgs.forEach(msg => {
        const isSent = msg.sender_id === currentUserId;
        const div = document.createElement('div');
        div.className = `flex ${isSent ? 'justify-end' : 'justify-start'}`;

        const time = formatTime(msg.created_at);
        const status = msg.status || 'sent';

        div.innerHTML = `
            <div class="max-w-xs md:max-w-md lg:max-w-lg">
                ${!isSent && currentConversation?.conversation_type === 'group' ?
                    `<p class="text-xs text-gray-500 mb-1">${escapeHtml(msg.sender_name || 'Unknown')}</p>` : ''}
                <div class="${isSent ? 'message-bubble-sent text-white' : 'message-bubble-received text-gray-800'} px-4 py-2 rounded-2xl ${isSent ? 'rounded-br-md' : 'rounded-bl-md'}">
                    <p>${escapeHtml(msg.content)}</p>
                </div>
                <div class="flex items-center mt-1 ${isSent ? 'justify-end' : ''}">
                    <span class="text-xs text-gray-500">${time}</span>
                    ${isSent ? `<span class="text-xs text-gray-500 ml-1">${getStatusIcon(status)}</span>` : ''}
                </div>
            </div>
        `;

        list.appendChild(div);
    });

    list.scrollTop = list.scrollHeight;
}

function addMessageToUI(message) {
    if (!currentConversation || currentConversation.id !== message.conversation_id) {
        return;
    }

    if (!messages[message.conversation_id]) {
        messages[message.conversation_id] = [];
    }

    const existingIndex = messages[message.conversation_id].findIndex(m => m.temp_id && m.temp_id === message.temp_id);
    if (existingIndex >= 0) {
        messages[message.conversation_id][existingIndex] = message;
    } else {
        const existingById = messages[message.conversation_id].findIndex(m => m.id === message.id);
        if (existingById < 0) {
            messages[message.conversation_id].push(message);
        }
    }

    renderMessages();
}

function updateConversationLastMessage(message) {
    const conv = conversations.find(c => c.id === message.conversation_id);
    if (conv) {
        conv.last_message = message;
        conv.updated_at = message.created_at;
        renderConversations();
    }
}

function sendMessage() {
    const input = document.getElementById('message-input');
    const content = input.value.trim();

    if (!content || !currentConversation) return;

    const tempId = 'temp-' + Date.now();
    const currentUserId = JSON.parse(localStorage.getItem('user'))?.id;

    const tempMessage = {
        id: tempId,
        temp_id: tempId,
        conversation_id: currentConversation.id,
        sender_id: currentUserId,
        content,
        created_at: new Date().toISOString(),
        status: 'sending',
    };

    addMessageToUI(tempMessage);

    const seq = wsClient.sendMessage(currentConversation.id, content, tempId);
    pendingMessages.set(seq, tempId);

    input.value = '';
}

function handleKeyPress(event) {
    if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        sendMessage();
    }
}

let typingTimeout;
function handleTyping() {
    if (!currentConversation) return;

    clearTimeout(typingTimeout);
    typingTimeout = setTimeout(() => {
        wsClient.sendTyping(currentConversation.id);
    }, 300);
}

function showTypingIndicator(userName) {
    const indicator = document.getElementById('typing-indicator');
    document.getElementById('typing-user').textContent = userName;
    indicator.classList.remove('hidden');

    setTimeout(() => {
        indicator.classList.add('hidden');
    }, 3000);
}

function updateOnlineStatus(userId, isOnline) {
    conversations.forEach(conv => {
        if (conv.other_user_id === userId) {
            conv.is_online = isOnline;
        }
    });
    renderConversations();

    if (currentConversation && currentConversation.other_user_id === userId) {
        document.getElementById('chat-status').textContent = isOnline ? 'Online' : 'Offline';
        document.getElementById('chat-online-status').classList.toggle('hidden', !isOnline);
    }
}

function switchTab(tab) {
    currentTab = tab;

    document.getElementById('tab-chats').className = `flex-1 py-3 text-center font-medium ${tab === 'chats' ? 'text-purple-600 border-b-2 border-purple-600' : 'text-gray-500 hover:text-gray-700'}`;
    document.getElementById('tab-groups').className = `flex-1 py-3 text-center font-medium ${tab === 'groups' ? 'text-purple-600 border-b-2 border-purple-600' : 'text-gray-500 hover:text-gray-700'}`;

    if (tab === 'groups') {
        loadGroups();
    } else {
        renderConversations();
    }
}

function toggleSidebar() {
    document.getElementById('sidebar').classList.toggle('sidebar-hidden');
}

function closeChat() {
    document.getElementById('sidebar').classList.remove('sidebar-hidden');
}

function showNewConversationModal() {
    document.getElementById('new-conversation-modal').classList.remove('hidden');
}

function hideNewConversationModal() {
    document.getElementById('new-conversation-modal').classList.add('hidden');
    document.getElementById('new-conv-username').value = '';
}

async function handleCreateConversation() {
    const username = document.getElementById('new-conv-username').value.trim();
    if (!username) return;

    try {
        const userResponse = await searchUsers(username);
        if (userResponse.users && userResponse.users.length > 0) {
            const userId = userResponse.users[0].id;

            const friendsResponse = await getFriends();
            const isFriend = friendsResponse.friends?.some(f => f.friend_id === userId);

            if (!isFriend) {
                alert('You can only start a conversation with friends. Add this user as a friend first!');
                window.location.href = '/friends.html';
                return;
            }

            await createConversation([userId]);
            hideNewConversationModal();
            await loadConversations();
        } else {
            alert('User not found');
        }
    } catch (error) {
        console.error('Failed to create conversation:', error);
        if (error.message.includes('not friends') || error.message.includes('friend')) {
            alert('You can only start a conversation with friends. Add this user as a friend first!');
        } else {
            alert('Failed to create conversation: ' + error.message);
        }
    }
}

function showCreateGroupModal() {
    document.getElementById('create-group-modal').classList.remove('hidden');
}

function hideCreateGroupModal() {
    document.getElementById('create-group-modal').classList.add('hidden');
    document.getElementById('group-name').value = '';
}

async function createGroup() {
    const name = document.getElementById('group-name').value.trim();
    if (!name) return;

    try {
        await createGroupApi(name, []);
        hideCreateGroupModal();
        await loadGroups();
        switchTab('groups');
    } catch (error) {
        console.error('Failed to create group:', error);
        alert('Failed to create group');
    }
}

function getInitials(name) {
    if (!name) return '?';
    return name
        .split(' ')
        .map(word => word[0])
        .join('')
        .toUpperCase()
        .slice(0, 2);
}

function formatTime(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (diff < 604800000) return date.toLocaleDateString([], { weekday: 'short' });
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function getStatusIcon(status) {
    switch (status) {
        case 'sending': return '🕐';
        case 'sent': return '✓';
        case 'delivered': return '✓✓';
        case 'read': return '<span class="text-blue-500">✓✓</span>';
        default: return '';
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

window.addEventListener('beforeunload', () => {
    wsClient.disconnect();
});

function handleLogout() {
    api.clearToken();
    wsClient.disconnect();
    window.location.href = '/';
}
