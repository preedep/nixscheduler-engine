// script.js
const expandedGroups = new Set();

async function fetchTasks() {
    try {
        const res = await fetch('/api/jobs');
        const tasks = await res.json();
        const tbody = document.querySelector('#task-table tbody');
        tbody.innerHTML = '';

        const filterText = document.getElementById('filter-input').value.toLowerCase();
        const filtered = tasks.filter(({ name, task_type }) =>
            name.toLowerCase().includes(filterText) || task_type.toLowerCase().includes(filterText)
        );

        document.getElementById('task-count').textContent = `${filtered.length} tasks`;

        const grouped = filtered.reduce((acc, task) => {
            (acc[task.name] ||= []).push(task);
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
                const key = status.toLowerCase();
                acc[key] = (acc[key] || 0) + 1;
                return acc;
            }, {});

            const chartHtml = Object.entries(statusCounts)
                .map(([status, count]) => `<span class="status-badge ${status}">${statusMap[status] || status} (${count})</span>`)
                .join(' ');

            tbody.insertAdjacentHTML('beforeend', `
                <tr class="group-header">
                    <td colspan="6">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div><strong>${name}</strong> ${chartHtml}</div>
                            <button onclick="toggleGroup('${toggleId}', this)">${isOpen ? '▼' : '▶'}</button>
                        </div>
                    </td>
                </tr>
            `);

            jobs.sort((a, b) => new Date(b.last_run || 0) - new Date(a.last_run || 0))
                .forEach(({ task_type, status = 'unknown', last_run, payload, message, execution_count = 0 }) => {
                    const displayStatus = statusMap[status.toLowerCase()] || status;
                    const errorMessageHtml = status.toLowerCase() === 'failed' && message
                        ? `<div class="error-message">⚠️ ${message}</div>`
                        : '';

                    tbody.insertAdjacentHTML('beforeend', `
                        <tr class="${toggleId}" style="display: ${isOpen ? '' : 'none'};">
                            <td></td>
                            <td>${task_type}</td>
                            <td><span class="status ${status.toLowerCase()}">${displayStatus}</span></td>
                            <td>${last_run || '-'}</td>
                            <td>
                                <pre>${JSON.stringify(payload, null, 2)}</pre>
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
        tbody.innerHTML = `<tr><td colspan="6" style="color:red;">Failed to load tasks.</td></tr>`;
    }
}

function toggleGroup(groupClass, btn) {
    const rows = document.querySelectorAll(`.${groupClass}`);
    const shouldShow = rows[0]?.style.display === 'none';
    rows.forEach(r => r.style.display = shouldShow ? '' : 'none');
    btn.textContent = shouldShow ? '▼' : '▶';
    shouldShow ? expandedGroups.add(groupClass) : expandedGroups.delete(groupClass);
}

function setupFilterBar() {
    document.querySelector('main').insertAdjacentHTML('afterbegin', `
        <div style="margin-bottom: 1rem; display: flex; justify-content: space-between; align-items: center;">
            <input id="filter-input" type="text" placeholder="Filter by name or type..." style="padding: 0.5rem; width: 300px; font-size: 1rem;">
            <span id="task-count" style="font-weight: bold;"></span>
        </div>
    `);
    document.getElementById('filter-input').addEventListener('input', fetchTasks);
}

document.addEventListener('DOMContentLoaded', () => {
    setupFilterBar();
    fetchTasks();
    setInterval(fetchTasks, 10000); // Refresh every 10 seconds
});