function showError(message) {
    const errorDiv = document.getElementById('error-message');
    errorDiv.textContent = message;
    errorDiv.classList.remove('hidden');
    setTimeout(() => {
        errorDiv.classList.add('hidden');
    }, 5000);
}

function showLogin() {
    document.getElementById('login-form').classList.remove('hidden');
    document.getElementById('register-form').classList.add('hidden');
}

function showRegister() {
    document.getElementById('login-form').classList.add('hidden');
    document.getElementById('register-form').classList.remove('hidden');
}

async function handleLogin() {
    const username = document.getElementById('username').value.trim();
    const password = document.getElementById('password').value;

    if (!username || !password) {
        showError('Please fill in all fields');
        return;
    }

    try {
        const response = await login(username, password);
        if (response) {
            window.location.href = '/chat.html';
        }
    } catch (error) {
        showError(error.message || 'Login failed');
    }
}

async function handleRegister() {
    const username = document.getElementById('reg-username').value.trim();
    const email = document.getElementById('reg-email').value.trim();
    const password = document.getElementById('reg-password').value;
    const confirmPassword = document.getElementById('reg-confirm-password').value;

    if (!username || !email || !password || !confirmPassword) {
        showError('Please fill in all fields');
        return;
    }

    if (password !== confirmPassword) {
        showError('Passwords do not match');
        return;
    }

    if (password.length < 6) {
        showError('Password must be at least 6 characters');
        return;
    }

    try {
        await register(username, email, password);
        showLogin();
        alert('Registration successful! Please login.');
    } catch (error) {
        showError(error.message || 'Registration failed');
    }
}

function handleLogout() {
    api.clearToken();
    window.location.href = '/';
}

document.addEventListener('DOMContentLoaded', () => {
    const token = localStorage.getItem('access_token');
    if (token && window.location.pathname === '/') {
        window.location.href = '/chat.html';
    }
});

document.getElementById('password')?.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        handleLogin();
    }
});

document.getElementById('reg-confirm-password')?.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        handleRegister();
    }
});
