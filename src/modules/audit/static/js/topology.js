// Topology Visualization JavaScript

document.addEventListener('DOMContentLoaded', function () {
    // Load topology data
    loadTopologyData();
});

// Load topology data
function loadTopologyData() {
    // Fetch topology
    fetch('/api/topology')
        .then(response => response.json())
        .then(topology => {
            renderTopology(topology);
            updateConnectivityStatus(topology);
            loadDependenciesTable(topology);
        })
        .catch(error => {
            console.error('Error loading topology:', error);
            showError('Failed to load topology data');
        });
}

// Render topology visualization
function renderTopology(topology) {
    const container = document.getElementById('topology-visualization');
    container.innerHTML = '';

    // Create SVG element
    const width = container.clientWidth;
    const height = 500;

    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);

    // Create a force simulation
    const simulation = d3.forceSimulation(topology.nodes)
        .force('link', d3.forceLink(topology.edges)
            .id(d => d.service_type)
            .distance(150))
        .force('charge', d3.forceManyBody().strength(-500))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(50));

    // Define color scale for service status
    const statusColor = {
        'Running': '#28a745',
        'Failed': '#dc3545',
        'NotStarted': '#6c757d',
        'Starting': '#ffc107',
        'ShuttingDown': '#17a2b8',
        'Stopped': '#343a40'
    };

    // Create arrow markers for directed edges
    svg.append('defs').selectAll('marker')
        .data(['connected', 'disconnected'])
        .enter().append('marker')
        .attr('id', d => d)
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 25)
        .attr('refY', 0)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('fill', d => d === 'connected' ? '#28a745' : '#dc3545')
        .attr('d', 'M0,-5L10,0L0,5');

    // Create links
    const links = svg.append('g')
        .selectAll('path')
        .data(topology.edges)
        .enter()
        .append('path')
        .attr('stroke', d => d.connected ? '#28a745' : '#dc3545')
        .attr('stroke-width', 2)
        .attr('fill', 'none')
        .attr('marker-end', d => d.connected ? 'url(#connected)' : 'url(#disconnected)');

    // Create node groups
    const nodes = svg.append('g')
        .selectAll('g')
        .data(topology.nodes)
        .enter()
        .append('g')
        .attr('class', 'node')
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended));

    // Add circles to nodes
    nodes.append('circle')
        .attr('r', 20)
        .attr('fill', d => statusColor[d.status] || '#6c757d')
        .attr('stroke', '#fff')
        .attr('stroke-width', 2);

    // Add labels to nodes
    nodes.append('text')
        .attr('dy', 30)
        .attr('text-anchor', 'middle')
        .text(d => d.service_type)
        .attr('font-size', '12px')
        .attr('font-weight', 'bold');

    // Add tooltips
    nodes.append('title')
        .text(d => `${d.service_type}\nStatus: ${d.status}`);

    // Update positions on simulation tick
    simulation.on('tick', () => {
        // Update link paths
        links.attr('d', d => {
            const dx = d.target.x - d.source.x;
            const dy = d.target.y - d.source.y;
            const dr = Math.sqrt(dx * dx + dy * dy);

            // Create curved paths
            return `M${d.source.x},${d.source.y}A${dr},${dr} 0 0,1 ${d.target.x},${d.target.y}`;
        });

        // Update node positions
        nodes.attr('transform', d => {
            // Keep nodes within bounds
            d.x = Math.max(20, Math.min(width - 20, d.x));
            d.y = Math.max(20, Math.min(height - 30, d.y));

            return `translate(${d.x},${d.y})`;
        });
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

// Update connectivity status
function updateConnectivityStatus(topology) {
    // Calculate connectivity percentage
    let connectedCount = 0;
    let totalEdges = topology.edges.length;

    for (const edge of topology.edges) {
        if (edge.connected) {
            connectedCount++;
        }
    }

    const connectivityPercentage = totalEdges > 0
        ? Math.round((connectedCount / totalEdges) * 100)
        : 0;

    // Update progress bar
    const progressBar = document.getElementById('connectivity-progress');
    progressBar.style.width = `${connectivityPercentage}%`;
    progressBar.setAttribute('aria-valuenow', connectivityPercentage);
    progressBar.textContent = `${connectivityPercentage}%`;

    // Set color based on percentage
    if (connectivityPercentage >= 90) {
        progressBar.classList.add('bg-success');
    } else if (connectivityPercentage >= 70) {
        progressBar.classList.add('bg-info');
    } else if (connectivityPercentage >= 50) {
        progressBar.classList.add('bg-warning');
    } else {
        progressBar.classList.add('bg-danger');
    }
}

// Load dependencies table
function loadDependenciesTable(topology) {
    const tableBody = document.getElementById('dependencies-table');
    tableBody.innerHTML = '';

    // Create a map of outgoing edges for each service
    const outgoingEdges = {};
    for (const edge of topology.edges) {
        if (!outgoingEdges[edge.source]) {
            outgoingEdges[edge.source] = [];
        }
        outgoingEdges[edge.source].push(edge);
    }

    // Create a map of service status
    const serviceStatus = {};
    for (const node of topology.nodes) {
        serviceStatus[node.service_type] = node.status;
    }

    // Add a row for each service
    for (const node of topology.nodes) {
        const tr = document.createElement('tr');

        // Service name
        const tdService = document.createElement('td');
        tdService.textContent = node.service_type;
        tr.appendChild(tdService);

        // Dependencies
        const tdDependencies = document.createElement('td');
        const dependencies = outgoingEdges[node.service_type] || [];

        if (dependencies.length === 0) {
            tdDependencies.textContent = 'None';
        } else {
            const ul = document.createElement('ul');
            ul.className = 'list-unstyled mb-0';

            for (const edge of dependencies) {
                const li = document.createElement('li');

                // Add connection status icon
                const icon = document.createElement('span');
                icon.className = edge.connected ? 'text-success' : 'text-danger';
                icon.textContent = edge.connected ? '✓ ' : '✗ ';
                li.appendChild(icon);

                // Add target service name
                li.appendChild(document.createTextNode(edge.target));

                ul.appendChild(li);
            }

            tdDependencies.appendChild(ul);
        }
        tr.appendChild(tdDependencies);

        // Service status
        const tdStatus = document.createElement('td');
        let statusText = '';
        let statusClass = '';

        switch (node.status) {
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
                statusText = node.status;
        }

        tdStatus.textContent = statusText;
        tdStatus.className = statusClass;
        tr.appendChild(tdStatus);

        tableBody.appendChild(tr);
    }
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