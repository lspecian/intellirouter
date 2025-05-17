// Dashboard JavaScript

document.addEventListener('DOMContentLoaded', function () {
    // Load dashboard data
    loadDashboardData();

    // Set up export button
    document.getElementById('export-button').addEventListener('click', exportReport);
});

// Load dashboard data
function loadDashboardData() {
    // Fetch current report
    fetch('/api/report')
        .then(response => response.json())
        .then(report => {
            updateSummary(report);
            loadTopologyPreview();
            loadServicesPreview(report);
            loadTestsPreview(report);
            loadErrorsPreview(report);
        })
        .catch(error => {
            console.error('Error loading report:', error);
            showError('Failed to load report data');
        });
}

// Update summary statistics
function updateSummary(report) {
    // Update status indicator
    const statusIndicator = document.getElementById('status-indicator');
    statusIndicator.innerHTML = report.success
        ? '<span class="badge bg-success">Success</span>'
        : '<span class="badge bg-danger">Failure</span>';

    // Update service count
    const servicesCount = document.getElementById('services-count');
    servicesCount.textContent = Object.keys(report.service_statuses).length;

    // Update test count
    const testsCount = document.getElementById('tests-count');
    testsCount.textContent = report.test_results.length;

    // Update communication tests count
    const commTestsCount = document.getElementById('comm-tests-count');
    commTestsCount.textContent = report.communication_tests.length;

    // Update metrics count
    const metricsCount = document.getElementById('metrics-count');
    metricsCount.textContent = report.metrics.length;

    // Update errors count
    const errorsCount = document.getElementById('errors-count');
    errorsCount.textContent = report.errors.length;

    // Add appropriate class based on error count
    if (report.errors.length > 0) {
        errorsCount.classList.add('text-danger');
        errorsCount.classList.add('fw-bold');
    } else {
        errorsCount.classList.add('text-success');
    }
}

// Load topology preview
function loadTopologyPreview() {
    fetch('/api/topology')
        .then(response => response.json())
        .then(topology => {
            renderTopologyPreview(topology);
        })
        .catch(error => {
            console.error('Error loading topology:', error);
            document.getElementById('topology-preview').innerHTML =
                '<div class="alert alert-danger">Failed to load topology data</div>';
        });
}

// Render topology preview
function renderTopologyPreview(topology) {
    const container = document.getElementById('topology-preview');
    container.innerHTML = '';

    // Create SVG element
    const width = container.clientWidth;
    const height = 300;

    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);

    // Create a simple force simulation
    const simulation = d3.forceSimulation(topology.nodes)
        .force('link', d3.forceLink(topology.edges).id(d => d.service_type))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2));

    // Define color scale for service status
    const statusColor = {
        'Running': '#28a745',
        'Failed': '#dc3545',
        'NotStarted': '#6c757d',
        'Starting': '#ffc107',
        'ShuttingDown': '#17a2b8',
        'Stopped': '#343a40'
    };

    // Create links
    const links = svg.append('g')
        .selectAll('line')
        .data(topology.edges)
        .enter()
        .append('line')
        .attr('stroke', d => d.connected ? '#28a745' : '#dc3545')
        .attr('stroke-width', 2);

    // Create nodes
    const nodes = svg.append('g')
        .selectAll('g')
        .data(topology.nodes)
        .enter()
        .append('g')
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended));

    // Add circles to nodes
    nodes.append('circle')
        .attr('r', 10)
        .attr('fill', d => statusColor[d.status] || '#6c757d');

    // Add labels to nodes
    nodes.append('text')
        .attr('dx', 12)
        .attr('dy', 4)
        .text(d => d.service_type);

    // Update positions on simulation tick
    simulation.on('tick', () => {
        links
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);

        nodes.attr('transform', d => `translate(${d.x},${d.y})`);
    });

    // Drag functions
    function dragstarted(event, d) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    function dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    function dragended(event, d) {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }
}

// Load services preview
function loadServicesPreview(report) {
    const container = document.getElementById('services-preview');
    container.innerHTML = '';

    // Create a table for services
    const table = document.createElement('table');
    table.className = 'table table-sm';

    // Create table header
    const thead = document.createElement('thead');
    thead.innerHTML = '<tr><th>Service</th><th>Status</th></tr>';
    table.appendChild(thead);

    // Create table body
    const tbody = document.createElement('tbody');

    // Add rows for each service
    for (const [service, status] of Object.entries(report.service_statuses)) {
        const tr = document.createElement('tr');

        // Service name
        const tdService = document.createElement('td');
        tdService.textContent = service;
        tr.appendChild(tdService);

        // Service status
        const tdStatus = document.createElement('td');
        let statusText = '';
        let statusClass = '';

        switch (status) {
            case 'Running':
                statusText = '✅ Running';
                statusClass = 'text-success';
                break;
            case 'Failed':
                statusText = '❌ Failed';
                statusClass = 'text-danger';
                break;
            case 'NotStarted':
                statusText = '⏳ Not Started';
                break;
            case 'Starting':
                statusText = '🔄 Starting';
                statusClass = 'text-warning';
                break;
            case 'ShuttingDown':
                statusText = '🔄 Shutting Down';
                statusClass = 'text-info';
                break;
            case 'Stopped':
                statusText = '⏹️ Stopped';
                statusClass = 'text-dark';
                break;
            default:
                statusText = status;
        }

        tdStatus.textContent = statusText;
        tdStatus.className = statusClass;
        tr.appendChild(tdStatus);

        tbody.appendChild(tr);
    }

    table.appendChild(tbody);
    container.appendChild(table);
}

// Load tests preview
function loadTestsPreview(report) {
    const container = document.getElementById('tests-preview');
    container.innerHTML = '';

    // Create a table for tests
    const table = document.createElement('table');
    table.className = 'table table-sm';

    // Create table header
    const thead = document.createElement('thead');
    thead.innerHTML = '<tr><th>Test</th><th>Status</th><th>Duration (ms)</th></tr>';
    table.appendChild(thead);

    // Create table body
    const tbody = document.createElement('tbody');

    // Add rows for each test (limit to 5)
    const tests = report.test_results.slice(0, 5);
    for (const test of tests) {
        const tr = document.createElement('tr');

        // Test name
        const tdTest = document.createElement('td');
        tdTest.textContent = test.test_flow;
        tr.appendChild(tdTest);

        // Test status
        const tdStatus = document.createElement('td');
        if (test.success) {
            tdStatus.innerHTML = '<span class="badge bg-success">Success</span>';
        } else {
            tdStatus.innerHTML = '<span class="badge bg-danger">Failure</span>';
        }
        tr.appendChild(tdStatus);

        // Test duration
        const tdDuration = document.createElement('td');
        tdDuration.textContent = test.duration_ms;
        tr.appendChild(tdDuration);

        tbody.appendChild(tr);
    }

    table.appendChild(tbody);
    container.appendChild(table);

    // Add message if there are more tests
    if (report.test_results.length > 5) {
        const moreMessage = document.createElement('p');
        moreMessage.className = 'text-muted text-center';
        moreMessage.textContent = `+ ${report.test_results.length - 5} more tests`;
        container.appendChild(moreMessage);
    }
}

// Load errors preview
function loadErrorsPreview(report) {
    const container = document.getElementById('errors-preview');
    container.innerHTML = '';

    if (report.errors.length === 0) {
        container.innerHTML = '<div class="alert alert-success">No errors detected</div>';
        return;
    }

    // Create a list for errors
    const ul = document.createElement('ul');
    ul.className = 'list-group';

    // Add items for each error (limit to 5)
    const errors = report.errors.slice(0, 5);
    for (const error of errors) {
        const li = document.createElement('li');
        li.className = 'list-group-item list-group-item-danger';
        li.textContent = error;
        ul.appendChild(li);
    }

    container.appendChild(ul);

    // Add message if there are more errors
    if (report.errors.length > 5) {
        const moreMessage = document.createElement('p');
        moreMessage.className = 'text-muted text-center mt-2';
        moreMessage.textContent = `+ ${report.errors.length - 5} more errors`;
        container.appendChild(moreMessage);
    }
}

// Export report
function exportReport() {
    const format = document.getElementById('export-format').value;
    const filename = document.getElementById('export-filename').value;

    // Validate filename
    if (!filename) {
        showError('Please enter a filename');
        return;
    }

    // Prepare export request
    const exportRequest = {
        format: format.toUpperCase(),
        filename: `${filename}.${format}`
    };

    // Send export request
    fetch('/api/export', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(exportRequest)
    })
        .then(response => response.json())
        .then(data => {
            if (data.success) {
                showSuccess(`Report exported successfully: ${data.message}`);

                // If download URL is provided, trigger download
                if (data.download_url) {
                    window.location.href = data.download_url;
                }
            } else {
                showError(`Export failed: ${data.message}`);
            }
        })
        .catch(error => {
            console.error('Error exporting report:', error);
            showError('Failed to export report');
        });
}

// Show error message
function showError(message) {
    // Create alert element
    const alert = document.createElement('div');
    alert.className = 'alert alert-danger alert-dismissible fade show';
    alert.setAttribute('role', 'alert');
    alert.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
    `;

    // Add to document
    document.body.insertBefore(alert, document.body.firstChild);

    // Auto-dismiss after 5 seconds
    setTimeout(() => {
        const bsAlert = new bootstrap.Alert(alert);
        bsAlert.close();
    }, 5000);
}

// Show success message
function showSuccess(message) {
    // Create alert element
    const alert = document.createElement('div');
    alert.className = 'alert alert-success alert-dismissible fade show';
    alert.setAttribute('role', 'alert');
    alert.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
    `;

    // Add to document
    document.body.insertBefore(alert, document.body.firstChild);

    // Auto-dismiss after 5 seconds
    setTimeout(() => {
        const bsAlert = new bootstrap.Alert(alert);
        bsAlert.close();
    }, 5000);
}