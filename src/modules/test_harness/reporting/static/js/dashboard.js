/**
 * IntelliRouter Test Dashboard
 * 
 * This file contains the JavaScript code for the test dashboard functionality.
 */

// Dashboard state
const dashboardState = {
    testRuns: [],
    metrics: {},
    trends: {
        passRateTrend: [],
        testCountTrend: [],
        durationTrend: [],
        failureTrend: [],
        flakyTestTrend: []
    },
    flakyTests: [],
    activeView: 'overview',
    refreshInterval: 30,
    timeRange: '1d',
    theme: 'light',
    charts: {},
    notifications: []
};

// Initialize dashboard
document.addEventListener('DOMContentLoaded', () => {
    initializeDashboard();
});

/**
 * Initialize the dashboard
 */
async function initializeDashboard() {
    // Set up event listeners
    setupEventListeners();

    // Load initial data
    await loadDashboardData();

    // Initialize charts
    initializeCharts();

    // Set up auto-refresh
    setupAutoRefresh();

    // Set up notifications
    setupNotifications();
}

/**
 * Set up event listeners
 */
function setupEventListeners() {
    // Navigation
    document.querySelectorAll('.dashboard-nav li').forEach(navItem => {
        navItem.addEventListener('click', () => {
            const view = navItem.getAttribute('data-view');
            switchView(view);
        });
    });

    // Refresh interval
    document.getElementById('refresh-interval').addEventListener('change', (e) => {
        dashboardState.refreshInterval = parseInt(e.target.value);
        setupAutoRefresh();
    });

    // Time range
    document.getElementById('time-range').addEventListener('change', (e) => {
        dashboardState.timeRange = e.target.value;
        loadDashboardData();
    });

    // Settings button
    document.getElementById('settings-btn').addEventListener('click', () => {
        openModal('settings-modal');
    });

    // Notifications button
    document.getElementById('notifications-btn').addEventListener('click', () => {
        openModal('notifications-modal');
    });

    // Close buttons for modals
    document.querySelectorAll('.close-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            closeModal(btn.closest('.modal').id);
        });
    });

    // Settings save button
    document.getElementById('settings-save').addEventListener('click', () => {
        saveSettings();
        closeModal('settings-modal');
    });

    // Settings cancel button
    document.getElementById('settings-cancel').addEventListener('click', () => {
        closeModal('settings-modal');
    });

    // Notifications clear button
    document.getElementById('notifications-clear').addEventListener('click', () => {
        clearNotifications();
    });

    // Notifications settings button
    document.getElementById('notifications-settings').addEventListener('click', () => {
        closeModal('notifications-modal');
        openModal('settings-modal');
    });

    // Export buttons
    document.querySelectorAll('[id^="export-"]').forEach(btn => {
        btn.addEventListener('click', () => {
            const view = btn.id.replace('export-', '');
            exportData(view);
        });
        /**
         * Switch to a different view
         * @param {string} view - The view to switch to
         */
        function switchView(view) {
            // Update active nav item
            document.querySelectorAll('.dashboard-nav li').forEach(navItem => {
                navItem.classList.remove('active');
            });
            document.querySelector(`.dashboard-nav li[data-view="${view}"]`).classList.add('active');

            // Update active view
            document.querySelectorAll('.dashboard-view').forEach(viewEl => {
                viewEl.classList.remove('active');
            });
            document.getElementById(view).classList.add('active');

            // Update state
            dashboardState.activeView = view;

            // Refresh charts in the active view
            refreshChartsInView(view);
        }

        /**
         * Refresh charts in the active view
         * @param {string} view - The view to refresh charts in
         */
        function refreshChartsInView(view) {
            // Resize charts to fit their containers
            Object.values(dashboardState.charts).forEach(chart => {
                if (chart.canvas.closest('.dashboard-view').id === view) {
                    chart.resize();
                }
            });
        }

        /**
         * Load dashboard data from the API
         */
        async function loadDashboardData() {
            try {
                // Show loading state
                showLoading(true);

                // Load test runs
                const testRunsResponse = await fetch('/api/test-runs');
                if (testRunsResponse.ok) {
                    dashboardState.testRuns = await testRunsResponse.json();
                }

                // Load metrics
                const metricsResponse = await fetch('/api/metrics');
                if (metricsResponse.ok) {
                    dashboardState.metrics = await metricsResponse.json();
                }

                // Load trends
                const trendsResponse = await fetch('/api/trends');
                if (trendsResponse.ok) {
                    dashboardState.trends = await trendsResponse.json();
                }

                // Load flaky tests
                const flakyTestsResponse = await fetch('/api/flaky-tests');
                if (flakyTestsResponse.ok) {
                    dashboardState.flakyTests = await flakyTestsResponse.json();
                }

                // Update UI
                updateDashboardUI();

                // Hide loading state
                showLoading(false);
            } catch (error) {
                console.error('Error loading dashboard data:', error);
                showError('Failed to load dashboard data. Please try again.');
                showLoading(false);
            }
        }

        /**
         * Update the dashboard UI with the latest data
         */
        function updateDashboardUI() {
            // Update metrics summary
            updateMetricsSummary();

            // Update recent test runs table
            updateRecentTestRunsTable();

            // Update all test runs table
            updateAllTestRunsTable();

            // Update top flaky tests table
            updateTopFlakyTestsTable();

            // Update flaky tests table
            updateFlakyTestsTable();

            // Update failures table
            updateFailuresTable();

            // Update charts
            updateCharts();
        }

        /**
         * Update the metrics summary
         */
        function updateMetricsSummary() {
            // Calculate total tests
            const totalTests = dashboardState.testRuns.reduce((total, run) => total + run.results.length, 0);
            document.getElementById('total-tests').textContent = totalTests;

            // Calculate pass rate
            const totalPassed = dashboardState.testRuns.reduce((total, run) => total + run.passed_count, 0);
            const passRate = totalTests > 0 ? (totalPassed / totalTests) * 100 : 0;
            document.getElementById('pass-rate').textContent = `${passRate.toFixed(2)}%`;

            // Calculate failed tests
            const totalFailed = dashboardState.testRuns.reduce((total, run) => total + run.failed_count, 0);
            document.getElementById('failed-tests').textContent = totalFailed;

            // Calculate flaky tests
            const flakyTestsCount = dashboardState.flakyTests.length;
            document.getElementById('flaky-tests-count').textContent = flakyTestsCount;
        }

        /**
         * Update the recent test runs table
         */
        function updateRecentTestRunsTable() {
            const tableBody = document.querySelector('#recent-test-runs tbody');
            tableBody.innerHTML = '';

            // Sort test runs by start time (descending)
            const sortedRuns = [...dashboardState.testRuns].sort((a, b) => {
                return new Date(b.start_time) - new Date(a.start_time);
            });

            // Take the 5 most recent runs
            const recentRuns = sortedRuns.slice(0, 5);

            // Add rows to the table
            recentRuns.forEach(run => {
                const row = document.createElement('tr');

                // Calculate pass rate
                const passRate = run.test_count > 0 ? (run.passed_count / run.test_count) * 100 : 0;

                // Format duration
                const duration = formatDuration(run.duration);

                row.innerHTML = `
            <td>${run.id}</td>
            <td>${run.name}</td>
            <td>${formatDate(run.start_time)}</td>
            <td>${duration}</td>
            <td>${run.test_count}</td>
            <td class="text-success">${run.passed_count}</td>
            <td class="text-danger">${run.failed_count}</td>
            <td class="text-warning">${run.skipped_count}</td>
            <td>${passRate.toFixed(2)}%</td>
        `;

                tableBody.appendChild(row);
            });
        }
    });

    // Test details close button
    document.getElementById('test-details-close').addEventListener('click', () => {
        closeModal('test-details-modal');
    });

    // Test details export button
    document.getElementById('test-details-export').addEventListener('click', () => {
        exportTestDetails();
    });

    // Close flaky test details button
    document.getElementById('close-flaky-test-details').addEventListener('click', () => {
        document.getElementById('flaky-test-details').style.display = 'none';
    });
}
/**
 * Update the all test runs table
 */
function updateAllTestRunsTable() {
    const tableBody = document.querySelector('#all-test-runs tbody');
    tableBody.innerHTML = '';

    // Sort test runs by start time (descending)
    const sortedRuns = [...dashboardState.testRuns].sort((a, b) => {
        return new Date(b.start_time) - new Date(a.start_time);
    });

    // Add rows to the table
    sortedRuns.forEach(run => {
        const row = document.createElement('tr');

        // Calculate pass rate
        const passRate = run.test_count > 0 ? (run.passed_count / run.test_count) * 100 : 0;

        // Format duration
        const duration = formatDuration(run.duration);

        row.innerHTML = `
            <td>${run.id}</td>
            <td>${run.name}</td>
            <td>${formatDate(run.start_time)}</td>
            <td>${formatDate(run.end_time)}</td>
            <td>${duration}</td>
            <td>${run.test_count}</td>
            <td class="text-success">${run.passed_count}</td>
            <td class="text-danger">${run.failed_count}</td>
            <td class="text-warning">${run.skipped_count}</td>
            <td>${passRate.toFixed(2)}%</td>
            <td class="actions">
                <button class="action-btn" data-run-id="${run.id}" title="View Details">
                    <i class="fas fa-eye"></i>
                </button>
                <button class="action-btn" data-run-id="${run.id}" title="Export">
                    <i class="fas fa-download"></i>
                </button>
            </td>
        `;

        tableBody.appendChild(row);
    });

    // Add event listeners to action buttons
    document.querySelectorAll('#all-test-runs .action-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const runId = e.currentTarget.getAttribute('data-run-id');
            const action = e.currentTarget.title;

            if (action === 'View Details') {
                viewTestRunDetails(runId);
            } else if (action === 'Export') {
                exportTestRun(runId);
            }
        });
    });
}

/**
 * Initialize charts
 */
function initializeCharts() {
    // Pass rate chart
    const passRateCtx = document.getElementById('pass-rate-chart').getContext('2d');
    dashboardState.charts.passRate = new Chart(passRateCtx, {
        type: 'line',
        data: {
            datasets: [{
                label: 'Pass Rate',
                data: [],
                borderColor: '#2ecc71',
                backgroundColor: 'rgba(46, 204, 113, 0.2)',
                borderWidth: 2,
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    type: 'time',
                    time: {
                        unit: 'day'
                    },
                    title: {
                        display: true,
                        text: 'Date'
                    }
                },
                y: {
                    min: 0,
                    max: 100,
                    title: {
                        display: true,
                        text: 'Pass Rate (%)'
                    }
                }
            },
            plugins: {
                tooltip: {
                    callbacks: {
                        label: (context) => `Pass Rate: ${context.parsed.y.toFixed(2)}%`
                    }
                }
            }
        }
    });

    // Test status chart
    const testStatusCtx = document.getElementById('test-status-chart').getContext('2d');
    dashboardState.charts.testStatus = new Chart(testStatusCtx, {
        type: 'pie',
        data: {
            labels: ['Passed', 'Failed', 'Skipped'],
            datasets: [{
                data: [0, 0, 0],
                backgroundColor: ['#2ecc71', '#e74c3c', '#f39c12']
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'right'
                }
            }
        }
    });

    // Initialize trends charts
    initializeTrendsCharts();
}

/**
 * Update charts with the latest data
 */
function updateCharts() {
    // Update pass rate chart
    const passRateData = dashboardState.trends.pass_rate_trend.map(([timestamp, value]) => ({
        x: new Date(timestamp),
        y: value * 100
    }));
    dashboardState.charts.passRate.data.datasets[0].data = passRateData;
    dashboardState.charts.passRate.update();

    // Update test status chart
    const totalPassed = dashboardState.testRuns.reduce((total, run) => total + run.passed_count, 0);
    const totalFailed = dashboardState.testRuns.reduce((total, run) => total + run.failed_count, 0);
    const totalSkipped = dashboardState.testRuns.reduce((total, run) => total + run.skipped_count, 0);
    dashboardState.charts.testStatus.data.datasets[0].data = [totalPassed, totalFailed, totalSkipped];
    dashboardState.charts.testStatus.update();

    // Update trends charts
    updateTrendsCharts();
}

/**
 * Set up auto-refresh
 */
function setupAutoRefresh() {
    // Clear existing interval
    if (dashboardState.refreshIntervalId) {
        clearInterval(dashboardState.refreshIntervalId);
    }

    // Set up new interval if refresh is enabled
    if (dashboardState.refreshInterval > 0) {
        dashboardState.refreshIntervalId = setInterval(() => {
            loadDashboardData();
        }, dashboardState.refreshInterval * 1000);
    }
}