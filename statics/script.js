const expandedGroups = new Set();

async function fetchTasks() {
    const res = await fetch('/api/jobs');
    const tasks = await res.json();
    const tbody = document.querySelector('#task-table tbody');
    tbody.innerHTML = '';

    const filterText = document.getElementById('filter-input').value.toLowerCase();
    const filtered = tasks.filter(t =>
        t.name.toLowerCase().includes(filterText) ||
        t.task_type.toLowerCase().includes(filterText)
    );

    document.getElementById('task-count').textContent = `${filtered.length} tasks`;

    const grouped = {};
    filtered.forEach(task => {
        if (!grouped[task.name]) grouped[task.name] = [];
        grouped[task.name].push(task);
    });

    const statusMap = {
        start: { label: '🟡 Start', class: 'start' },
        scheduled: { label: '📅 Scheduled', class: 'scheduled' },
        running: { label: '🔄 Running', class: 'running' },
        success: { label: '✅ Success', class: 'success' },
        failed: { label: '❌ Failed', class: 'failed' },
        disabled: { label: '🚫 Disabled', class: 'disabled' },
    };

    Object.entries(grouped).forEach(([name, jobs], idx) => {
        const tr = document.createElement('tr');
        const toggleId = `group-${idx}`;
        const isOpen = expandedGroups.has(toggleId);

        tr.innerHTML = `
      <td colspan="6">
        <strong>${name}</strong>
        <button onclick="toggleGroup('${toggleId}', this)">
          ${isOpen ? '▼' : '▶'}
        </button>
      </td>
    `;
        tbody.appendChild(tr);

        jobs
            .sort((a, b) => {
                const dateA = a.last_run ? new Date(a.last_run) : new Date(0);
                const dateB = b.last_run ? new Date(b.last_run) : new Date(0);
                return dateB - dateA;
            })
            .forEach((task, i) => {
                const row = document.createElement('tr');
                const statusInfo = statusMap[task.status] || { label: task.status, class: 'unknown' };
                const execCount = task.execution_count || 0;

                row.className = toggleId;
                row.style.display = isOpen ? '' : 'none';

                row.innerHTML = `
          <td></td>
          <td>${task.task_type}</td>
          <td><span class="status ${statusInfo.class}">${statusInfo.label}</span></td>
          <td>${task.last_run || '-'}</td>
          <td><pre>${JSON.stringify(task.payload, null, 2)}</pre></td>
          <td>${execCount}</td>
        `;

                tbody.appendChild(row);
            });
    });
}

function toggleGroup(groupClass, btn) {
    const rows = document.querySelectorAll(`.${groupClass}`);
    const currentlyExpanded = rows[0]?.style.display !== 'none';
    const shouldShow = !currentlyExpanded;

    rows.forEach(r => {
        r.style.display = shouldShow ? '' : 'none';
    });

    if (shouldShow) {
        expandedGroups.add(groupClass);
        btn.textContent = '▼';
    } else {
        expandedGroups.delete(groupClass);
        btn.textContent = '▶';
    }
}

function setupFilterBar() {
    const filter = document.createElement('div');
    filter.innerHTML = `
    <div style="margin-bottom: 1rem; display: flex; justify-content: space-between; align-items: center;">
      <input id="filter-input" type="text" placeholder="Filter by name or type..." style="padding: 0.5rem; width: 300px; font-size: 1rem;">
      <span id="task-count" style="font-weight: bold;"></span>
    </div>
  `;
    document.querySelector('main').insertBefore(filter, document.querySelector('section'));
    document.getElementById('filter-input').addEventListener('input', fetchTasks);
}

setupFilterBar();
fetchTasks();
setInterval(fetchTasks, 10000);
