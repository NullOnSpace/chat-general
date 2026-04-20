let currentTab = 'friends';
let friends = [];
let requests = [];
let sentRequests = [];

document.addEventListener('DOMContentLoaded', async () => {
    const token = localStorage.getItem('access_token');
    if (!token) {
        window.location.href = '/';
        return;
    }

    await loadAllData();
});

async function loadAllData() {
    await Promise.all([
        loadFriends(),
        loadRequests(),
        loadSentRequests()
    ]);
    updateRequestBadge();
}

async function loadFriends() {
    try {
        const response = await getFriends();
        friends = response.friends || [];
        renderFriends();
    } catch (error) {
        console.error('Failed to load friends:', error);
    }
}

async function loadRequests() {
    try {
        const response = await getFriendRequests();
        requests = response.requests || [];
        renderRequests();
    } catch (error) {
        console.error('Failed to load requests:', error);
    }
}

async function loadSentRequests() {
    try {
        const response = await getSentFriendRequests();
        sentRequests = response.requests || [];
        renderSentRequests();
    } catch (error) {
        console.error('Failed to load sent requests:', error);
    }
}

function updateRequestBadge() {
    const pendingCount = requests.filter(r => r.status === 'pending').length;
    const badge = document.getElementById('request-badge');
    if (pendingCount > 0) {
        badge.textContent = pendingCount;
        badge.classList.remove('hidden');
    } else {
        badge.classList.add('hidden');
    }
}

function switchTab(tab) {
    currentTab = tab;

    document.getElementById('tab-friends').className = `flex-1 py-3 text-center font-medium ${tab === 'friends' ? 'tab-active' : 'text-gray-500 hover:text-gray-700'}`;
    document.getElementById('tab-requests').className = `flex-1 py-3 text-center font-medium ${tab === 'requests' ? 'tab-active' : 'text-gray-500 hover:text-gray-700'}`;
    document.getElementById('tab-sent').className = `flex-1 py-3 text-center font-medium ${tab === 'sent' ? 'tab-active' : 'text-gray-500 hover:text-gray-700'}`;

    document.getElementById('friends-list').classList.toggle('hidden', tab !== 'friends');
    document.getElementById('requests-list').classList.toggle('hidden', tab !== 'requests');
    document.getElementById('sent-list').classList.toggle('hidden', tab !== 'sent');
}

function renderFriends() {
    const list = document.getElementById('friends-list');

    if (friends.length === 0) {
        list.innerHTML = `
            <div class="p-8 text-center text-gray-500">
                <i class="fas fa-user-friends text-4xl mb-3"></i>
                <p>No friends yet</p>
                <p class="text-sm mt-1">Add friends to start chatting!</p>
            </div>
        `;
        return;
    }

    list.innerHTML = friends.map(friend => `
        <div class="p-4 flex items-center justify-between hover:bg-gray-50">
            <div class="flex items-center">
                <div class="w-12 h-12 rounded-full bg-gradient-to-r from-purple-600 to-indigo-600 flex items-center justify-center text-white font-semibold">
                    ${getInitials(friend.remark || friend.friend_id)}
                </div>
                <div class="ml-3">
                    <h4 class="font-medium text-gray-800">${friend.remark || 'Friend'}</h4>
                    <p class="text-sm text-gray-500">${formatDate(friend.created_at)}</p>
                </div>
            </div>
            <div class="flex items-center space-x-2">
                <button onclick="startChat('${friend.friend_id}')" class="p-2 text-purple-600 hover:bg-purple-100 rounded-lg transition" title="Start Chat">
                    <i class="fas fa-comment"></i>
                </button>
                <button onclick="confirmDeleteFriend('${friend.friend_id}')" class="p-2 text-red-600 hover:bg-red-100 rounded-lg transition" title="Remove Friend">
                    <i class="fas fa-user-minus"></i>
                </button>
            </div>
        </div>
    `).join('');
}

function renderRequests() {
    const list = document.getElementById('requests-list');

    if (requests.length === 0) {
        list.innerHTML = `
            <div class="p-8 text-center text-gray-500">
                <i class="fas fa-inbox text-4xl mb-3"></i>
                <p>No friend requests</p>
            </div>
        `;
        return;
    }

    list.innerHTML = requests.map(request => `
        <div class="p-4 flex items-center justify-between hover:bg-gray-50">
            <div class="flex items-center">
                <div class="w-12 h-12 rounded-full bg-gradient-to-r from-purple-600 to-indigo-600 flex items-center justify-center text-white font-semibold">
                    ${getInitials(request.from_user_id)}
                </div>
                <div class="ml-3">
                    <h4 class="font-medium text-gray-800">User</h4>
                    <p class="text-sm text-gray-500">${request.message || 'Wants to be your friend'}</p>
                    <p class="text-xs text-gray-400">${formatDate(request.created_at)}</p>
                </div>
            </div>
            <div class="flex items-center space-x-2">
                ${request.status === 'pending' ? `
                    <button onclick="handleAcceptRequest('${request.id}')" class="px-3 py-1 bg-green-600 text-white rounded-lg hover:bg-green-700 transition text-sm">
                        Accept
                    </button>
                    <button onclick="handleRejectRequest('${request.id}')" class="px-3 py-1 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition text-sm">
                        Decline
                    </button>
                ` : `
                    <span class="text-sm text-gray-500 capitalize">${request.status}</span>
                `}
            </div>
        </div>
    `).join('');
}

function renderSentRequests() {
    const list = document.getElementById('sent-list');

    if (sentRequests.length === 0) {
        list.innerHTML = `
            <div class="p-8 text-center text-gray-500">
                <i class="fas fa-paper-plane text-4xl mb-3"></i>
                <p>No sent requests</p>
            </div>
        `;
        return;
    }

    list.innerHTML = sentRequests.map(request => `
        <div class="p-4 flex items-center justify-between hover:bg-gray-50">
            <div class="flex items-center">
                <div class="w-12 h-12 rounded-full bg-gradient-to-r from-purple-600 to-indigo-600 flex items-center justify-center text-white font-semibold">
                    ${getInitials(request.to_user_id)}
                </div>
                <div class="ml-3">
                    <h4 class="font-medium text-gray-800">User</h4>
                    <p class="text-sm text-gray-500">${request.message || 'Friend request'}</p>
                    <p class="text-xs text-gray-400">${formatDate(request.created_at)}</p>
                </div>
            </div>
            <div class="flex items-center">
                <span class="px-3 py-1 rounded-full text-sm ${
                    request.status === 'pending' ? 'bg-yellow-100 text-yellow-700' :
                    request.status === 'accepted' ? 'bg-green-100 text-green-700' :
                    'bg-red-100 text-red-700'
                } capitalize">
                    ${request.status}
                </span>
            </div>
        </div>
    `).join('');
}

async function handleAcceptRequest(requestId) {
    try {
        await acceptFriendRequest(requestId);
        await loadAllData();
    } catch (error) {
        alert('Failed to accept request: ' + error.message);
    }
}

async function handleRejectRequest(requestId) {
    try {
        await rejectFriendRequest(requestId);
        await loadAllData();
    } catch (error) {
        alert('Failed to reject request: ' + error.message);
    }
}

function showAddFriendModal() {
    document.getElementById('add-friend-modal').classList.remove('hidden');
}

function hideAddFriendModal() {
    document.getElementById('add-friend-modal').classList.add('hidden');
    document.getElementById('add-friend-user-id').value = '';
    document.getElementById('add-friend-message').value = '';
}

async function sendNewFriendRequest() {
    const userId = document.getElementById('add-friend-user-id').value.trim();
    const message = document.getElementById('add-friend-message').value.trim();

    if (!userId) {
        alert('Please enter a user ID');
        return;
    }

    try {
        await sendFriendRequest(userId, message || null);
        hideAddFriendModal();
        await loadAllData();
        switchTab('sent');
    } catch (error) {
        alert('Failed to send request: ' + error.message);
    }
}

function confirmDeleteFriend(friendId) {
    document.getElementById('confirm-title').textContent = 'Remove Friend';
    document.getElementById('confirm-message').textContent = 'Are you sure you want to remove this friend?';
    document.getElementById('confirm-btn').onclick = async () => {
        try {
            await deleteFriend(friendId);
            hideConfirmModal();
            await loadAllData();
        } catch (error) {
            alert('Failed to remove friend: ' + error.message);
        }
    };
    document.getElementById('confirm-modal').classList.remove('hidden');
}

function hideConfirmModal() {
    document.getElementById('confirm-modal').classList.add('hidden');
}

function startChat(friendId) {
    localStorage.setItem('start_chat_with', friendId);
    window.location.href = '/chat.html';
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

function formatDate(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
}
