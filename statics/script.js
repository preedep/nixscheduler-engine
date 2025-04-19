const expandedGroups = new Set();

function escapeHtml(unsafe = '') {
    return unsafe.replace(/[&<>"']/g, match => {
        const escapes = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;',
        };
        return escapes[match];
    });
}

function checkLogin() {
    const cookies = document.cookie.split(';').map(c => c.trim());
    const isLoggedIn = cookies.some(c => c.startsWith('logged_in=') && c.split('=')[1] === 'true');
    if (!isLoggedIn) {
        window.location.href = '/login.html';
    }
}

function buildOverview(tasks) {
    const container = document.getElementById('job-overview');
    if (!container) return;

    container.innerHTML = '';
    const statusGroups = tasks.reduce((acc, t) => {
        const status = (t.status || 'unknown').toLowerCase();
        acc[status] = acc[status] || [];
        acc[status].push(t);
        return acc;
    }, {});
    Object.entries(statusGroups).forEach(([status, jobs]) => {
        const html = `
            <div class="card">
                <h3>${status.toUpperCase()}</h3>
                <p>${jobs.length} job(s)</p>
            </div>
        `;
        container.insertAdjacentHTML('beforeend', html);
    });
}

async function fetchTasks() {
    try {
        const res = await fetch('/api/jobs');
        if (res.status === 401) {
            window.location.href = '/login.html';
            return;
        }

        const tasks = await res.json();
        buildOverview(tasks);

        const tbody = document.querySelector('#task-table tbody');
        if (!tbody) return;

        tbody.innerHTML = '';

        const filterInput = document.getElementById('filter-input');
        const filterText = filterInput?.value.toLowerCase() || '';

        const filtered = tasks.filter(({ name = '', task_type = '' }) =>
            name.toLowerCase().includes(filterText) || task_type.toLowerCase().includes(filterText)
        );

        const taskCountEl = document.getElementById('task-count');
        if (taskCountEl) taskCountEl.textContent = `${filtered.length} tasks`;

        const grouped = filtered.reduce((acc, task) => {
            const key = task.name || 'Unknown';
            acc[key] = acc[key] || [];
            acc[key].push(task);
            return acc;
        }, {});

        const statusMap = {
            start: '🟡 Start',
            scheduled: '📅 Scheduled',
            running: '🔄 Running',
            success: '✅ Success',
            failed: '❌ Failed',
            disabled: '🚫 Disabled',
        };

        Object.entries(grouped).forEach(([name, jobs], idx) => {
            const toggleId = `group-${idx}`;
            const isOpen = expandedGroups.has(toggleId);

            const statusCounts = jobs.reduce((acc, { status = 'unknown' }) => {
                const key = (status || 'unknown').toLowerCase();
                acc[key] = (acc[key] || 0) + 1;
                return acc;
            }, {});

            const chartHtml = Object.entries(statusCounts)
                .map(([status, count]) =>
                    `<span class="status-badge ${status}">${statusMap[status] || status} (${count})</span>`
                ).join(' ');

            tbody.insertAdjacentHTML('beforeend', `
                <tr class="group-header">
                    <td colspan="6">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div><strong>${escapeHtml(name)}</strong> ${chartHtml}</div>
                            <button onclick="toggleGroup('${toggleId}', this)">${isOpen ? '▼' : '▶'}</button>
                        </div>
                    </td>
                </tr>
            `);

            jobs
                .sort((a, b) => new Date(b.last_run || 0) - new Date(a.last_run || 0))
                .forEach(({ task_type = '-', status = 'unknown', last_run, payload, message, execution_count = 0 }) => {
                    const statusKey = status.toLowerCase();
                    const displayStatus = statusMap[statusKey] || statusKey;
                    const safeMessage = message?.trim() ? escapeHtml(message.trim()) : '';
                    const errorMessageHtml = safeMessage
                        ? `<div class="error-message">💬 ${safeMessage}</div>`
                        : '';

                    tbody.insertAdjacentHTML('beforeend', `
                        <tr class="${toggleId}" style="display: ${isOpen ? '' : 'none'};">
                            <td></td>
                            <td>${escapeHtml(task_type)}</td>
                            <td><span class="status ${statusKey}">${displayStatus}</span></td>
                            <td>${last_run || '-'}</td>
                            <td>
                                <pre>${escapeHtml(JSON.stringify(payload, null, 2))}</pre>
                                ${errorMessageHtml}
                            </td>
                            <td>${execution_count}</td>
                        </tr>
                    `);
                });
        });
    } catch (error) {
        console.error('Error fetching tasks:', error);
        const tbody = document.querySelector('#task-table tbody');
        if (tbody) {
            tbody.innerHTML = '<tr><td colspan="6" style="color:red;">Failed to load tasks.</td></tr>';
        }
    }
}

function toggleGroup(groupClass, btn) {
    const rows = document.querySelectorAll(`.${groupClass}`);
    const shouldShow = rows[0]?.style.display === 'none';
    rows.forEach(r => r.style.display = shouldShow ? '' : 'none');
    btn.textContent = shouldShow ? '▼' : '▶';
    if (shouldShow) {
        expandedGroups.add(groupClass);
    } else {
        expandedGroups.delete(groupClass);
    }
}

function setupFilterBar() {
    const container = document.getElementById('filter-bar');
    if (!container) return;

    container.innerHTML = `
        <div style="margin-bottom: 1rem; display: flex; justify-content: space-between; align-items: center;">
            <input id="filter-input" type="text" placeholder="Filter by name or type..."
                style="padding: 0.5rem; width: 300px; font-size: 1rem;">
            <span id="task-count" style="font-weight: bold;"></span>
        </div>
    `;

    document.getElementById('filter-input')?.addEventListener('input', fetchTasks);
}

document.addEventListener('DOMContentLoaded', () => {
    checkLogin();
    setupFilterBar();
    fetchTasks();
    setInterval(fetchTasks, 10000);
});