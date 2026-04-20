const API_BASE = '/api/v1';

class ApiClient {
    constructor() {
        this.token = localStorage.getItem('access_token');
    }

    setToken(token) {
        this.token = token;
        localStorage.setItem('access_token', token);
    }

    clearToken() {
        this.token = null;
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('user');
    }

    getHeaders() {
        const headers = {
            'Content-Type': 'application/json',
        };
        if (this.token) {
            headers['Authorization'] = `Bearer ${this.token}`;
        }
        return headers;
    }

    async request(method, endpoint, body = null) {
        const options = {
            method,
            headers: this.getHeaders(),
        };

        if (body) {
            options.body = JSON.stringify(body);
        }

        const response = await fetch(`${API_BASE}${endpoint}`, options);

        if (response.status === 401) {
            const refreshed = await this.refreshToken();
            if (refreshed) {
                options.headers = this.getHeaders();
                const retryResponse = await fetch(`${API_BASE}${endpoint}`, options);
                return this.handleResponse(retryResponse);
            } else {
                this.clearToken();
                window.location.href = '/';
                return null;
            }
        }

        return this.handleResponse(response);
    }

    async handleResponse(response) {
        if (!response.ok) {
            const error = await response.json().catch(() => ({ message: 'Request failed' }));
            throw new Error(error.message || error.error || 'Request failed');
        }
        if (response.status === 204) {
            return null;
        }
        return response.json();
    }

    async refreshToken() {
        const refreshToken = localStorage.getItem('refresh_token');
        if (!refreshToken) return false;

        try {
            const response = await fetch(`${API_BASE}/auth/refresh`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ refresh_token: refreshToken }),
            });

            if (response.ok) {
                const data = await response.json();
                this.setToken(data.access_token);
                localStorage.setItem('refresh_token', data.refresh_token);
                return true;
            }
        } catch (e) {
            console.error('Token refresh failed:', e);
        }
        return false;
    }

    get(endpoint) {
        return this.request('GET', endpoint);
    }

    post(endpoint, body) {
        return this.request('POST', endpoint, body);
    }

    put(endpoint, body) {
        return this.request('PUT', endpoint, body);
    }

    delete(endpoint) {
        return this.request('DELETE', endpoint);
    }
}

const api = new ApiClient();

async function login(username, password) {
    const response = await api.post('/auth/login', { username, password });
    if (response) {
        api.setToken(response.access_token);
        localStorage.setItem('refresh_token', response.refresh_token);
        if (response.user) {
            localStorage.setItem('user', JSON.stringify(response.user));
        }
    }
    return response;
}

async function register(username, email, password) {
    return api.post('/auth/register', { username, email, password });
}

async function getConversations() {
    return api.get('/conversations');
}

async function createConversation(participantIds) {
    return api.post('/conversations', { participant_ids: participantIds });
}

async function getMessages(conversationId, limit = 50, before = null) {
    let url = `/conversations/${conversationId}/messages?limit=${limit}`;
    if (before) {
        url += `&before=${before}`;
    }
    return api.get(url);
}

async function sendMessageApi(conversationId, content) {
    return api.post('/messages', { conversation_id: conversationId, content });
}

async function getGroups() {
    return api.get('/groups');
}

async function createGroupApi(name, memberIds) {
    return api.post('/groups', { name, member_ids: memberIds });
}

async function getGroupMembers(groupId) {
    return api.get(`/groups/${groupId}/members`);
}

async function addGroupMember(groupId, userId) {
    return api.post(`/groups/${groupId}/members`, { user_id: userId });
}

async function removeGroupMember(groupId, userId) {
    return api.delete(`/groups/${groupId}/members/${userId}`);
}

async function searchUsers(query) {
    return api.get(`/users/search?q=${encodeURIComponent(query)}`);
}

async function getCurrentUser() {
    return api.get('/auth/me');
}

// Friend API
async function getFriends() {
    return api.get('/friends');
}

async function deleteFriend(friendId) {
    return api.delete(`/friends/${friendId}`);
}

async function getFriendRequests() {
    return api.get('/friends/requests');
}

async function getSentFriendRequests() {
    return api.get('/friends/requests/sent');
}

async function sendFriendRequest(toUserId, message = null) {
    return api.post('/friends/requests', { to_user_id: toUserId, message });
}

async function acceptFriendRequest(requestId) {
    return api.put(`/friends/requests/${requestId}/accept`);
}

async function rejectFriendRequest(requestId) {
    return api.delete(`/friends/requests/${requestId}/reject`);
}
