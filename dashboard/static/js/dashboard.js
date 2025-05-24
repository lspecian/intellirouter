/**
 * IntelliRouter Dashboard JavaScript
 * 
 * This file contains the JavaScript code for the IntelliRouter dashboard.
 */

document.addEventListener('DOMContentLoaded', function () {
    // Initialize tooltips
    const tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
    tooltipTriggerList.map(function (tooltipTriggerEl) {
        return new bootstrap.Tooltip(tooltipTriggerEl);
    });

    // Initialize popovers
    const popoverTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="popover"]'));
    popoverTriggerList.map(function (popoverTriggerEl) {
        return new bootstrap.Popover(popoverTriggerEl);
    });

    // Format dates
    const dateElements = document.querySelectorAll('.format-date');
    dateElements.forEach(function (element) {
        const timestamp = element.getAttribute('data-timestamp');
        if (timestamp) {
            const date = new Date(timestamp);
            element.textContent = formatDate(date);
        }
    });

    // Format relative times
    const relativeTimeElements = document.querySelectorAll('.format-relative-time');
    relativeTimeElements.forEach(function (element) {
        const timestamp = element.getAttribute('data-timestamp');
        if (timestamp) {
            const date = new Date(timestamp);
            element.textContent = formatRelativeTime(date);
        }
    });

    // Setup auto-refresh
    setupAutoRefresh();

    // Setup collapsible sections
    setupCollapsibleSections();

    // Setup chart resizing
    setupChartResizing();

    // Setup API polling
    setupApiPolling();
});

/**
 * Format a date as a human-readable string
 * @param {Date} date - The date to format
 * @returns {string} The formatted date string
 */
function formatDate(date) {
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}

/**
 * Format a date as a relative time string
 * @param {Date} date - The date to format
 * @returns {string} The formatted relative time string
 */
function formatRelativeTime(date) {
    const now = new Date();
    const diffMs = now - date;
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);

    if (diffDay > 0) {
        return diffDay + ' day' + (diffDay > 1 ? 's' : '') + ' ago';
    } else if (diffHour > 0) {
        return diffHour + ' hour' + (diffHour > 1 ? 's' : '') + ' ago';
    } else if (diffMin > 0) {
        return diffMin + ' minute' + (diffMin > 1 ? 's' : '') + ' ago';
    } else {
        return 'just now';
    }
}

/**
 * Setup auto-refresh functionality
 */
function setupAutoRefresh() {
    const refreshInterval = document.querySelector('meta[http-equiv="refresh"]');
    if (refreshInterval) {
        const interval = parseInt(refreshInterval.getAttribute('content'), 10);
        if (interval > 0) {
            const refreshButton = document.querySelector('.btn-refresh');
            if (refreshButton) {
                let countdown = interval;
                const updateCountdown = () => {
                    countdown--;
                    if (countdown <= 0) {
                        window.location.reload();
                    } else {
                        refreshButton.textContent = `Refresh (${countdown})`;
                        setTimeout(updateCountdown, 1000);
                    }
                };
                setTimeout(updateCountdown, 1000);
            }
        }
    }
}

/**
 * Setup collapsible sections
 */
function setupCollapsibleSections() {
    const collapsibleHeaders = document.querySelectorAll('.collapsible-header');
    collapsibleHeaders.forEach(function (header) {
        header.addEventListener('click', function () {
            const content = this.nextElementSibling;
            const icon = this.querySelector('.collapse-icon');

            if (content.style.maxHeight) {
                content.style.maxHeight = null;
                icon.classList.remove('bi-chevron-up');
                icon.classList.add('bi-chevron-down');
            } else {
                content.style.maxHeight = content.scrollHeight + 'px';
                icon.classList.remove('bi-chevron-down');
                icon.classList.add('bi-chevron-up');
            }
        });
    });
}

/**
 * Setup chart resizing
 */
function setupChartResizing() {
    window.addEventListener('resize', function () {
        if (window.Chart && window.Chart.instances) {
            for (const id in window.Chart.instances) {
                if (window.Chart.instances.hasOwnProperty(id)) {
                    window.Chart.instances[id].resize();
                }
            }
        }
    });
}

/**
 * Setup API polling
 */
function setupApiPolling() {
    const apiPollingElements = document.querySelectorAll('[data-api-poll]');
    apiPollingElements.forEach(function (element) {
        const endpoint = element.getAttribute('data-api-poll');
        const interval = parseInt(element.getAttribute('data-api-poll-interval'), 10) || 30000;

        if (endpoint) {
            const pollApi = () => {
                fetch(endpoint)
                    .then(response => response.json())
                    .then(data => {
                        updateElementWithApiData(element, data);
                    })
                    .catch(error => {
                        console.error('Error polling API:', error);
                    });
            };

            // Initial poll
            pollApi();

            // Setup interval
            setInterval(pollApi, interval);
        }
    });
}

/**
 * Update an element with API data
 * @param {HTMLElement} element - The element to update
 * @param {Object} data - The API data
 */
function updateElementWithApiData(element, data) {
    const updateType = element.getAttribute('data-api-update-type') || 'text';
    const dataPath = element.getAttribute('data-api-data-path') || '';

    // Get the data value using the path
    let value = data;
    if (dataPath) {
        const pathParts = dataPath.split('.');
        for (const part of pathParts) {
            if (value && typeof value === 'object' && part in value) {
                value = value[part];
            } else {
                value = null;
                break;
            }
        }
    }

    // Update the element based on the update type
    if (value !== null) {
        switch (updateType) {
            case 'text':
                element.textContent = value;
                break;
            case 'html':
                element.innerHTML = value;
                break;
            case 'value':
                element.value = value;
                break;
            case 'attribute':
                const attribute = element.getAttribute('data-api-attribute');
                if (attribute) {
                    element.setAttribute(attribute, value);
                }
                break;
            case 'chart':
                updateChart(element, value);
                break;
            case 'progress':
                updateProgressBar(element, value);
                break;
            case 'table':
                updateTable(element, value);
                break;
            default:
                console.warn('Unknown API update type:', updateType);
        }
    }
}

/**
 * Update a chart with new data
 * @param {HTMLElement} element - The chart element
 * @param {Object} data - The chart data
 */
function updateChart(element, data) {
    const chartId = element.id;
    const chart = Chart.getChart(chartId);

    if (chart) {
        if (data.labels) {
            chart.data.labels = data.labels;
        }

        if (data.datasets) {
            chart.data.datasets = data.datasets;
        }

        chart.update();
    }
}

/**
 * Update a progress bar with a new value
 * @param {HTMLElement} element - The progress bar element
 * @param {number} value - The new value
 */
function updateProgressBar(element, value) {
    const progressBar = element.querySelector('.progress-bar');
    if (progressBar) {
        progressBar.style.width = value + '%';
        progressBar.setAttribute('aria-valuenow', value);

        // Update text if needed
        if (progressBar.textContent.includes('%')) {
            progressBar.textContent = value + '%';
        }
    }
}

/**
 * Update a table with new data
 * @param {HTMLElement} element - The table element
 * @param {Array} data - The table data
 */
function updateTable(element, data) {
    const tbody = element.querySelector('tbody');
    if (tbody && Array.isArray(data)) {
        // Clear existing rows
        tbody.innerHTML = '';

        // Add new rows
        data.forEach(function (rowData) {
            const row = document.createElement('tr');

            if (Array.isArray(rowData)) {
                // Array of values
                rowData.forEach(function (cellData) {
                    const cell = document.createElement('td');
                    cell.textContent = cellData;
                    row.appendChild(cell);
                });
            } else if (typeof rowData === 'object') {
                // Object with keys
                const columns = element.getAttribute('data-api-columns');
                if (columns) {
                    const columnKeys = columns.split(',');
                    columnKeys.forEach(function (key) {
                        const cell = document.createElement('td');
                        cell.textContent = rowData[key.trim()] || '';
                        row.appendChild(cell);
                    });
                } else {
                    // Use all keys
                    for (const key in rowData) {
                        if (rowData.hasOwnProperty(key)) {
                            const cell = document.createElement('td');
                            cell.textContent = rowData[key];
                            row.appendChild(cell);
                        }
                    }
                }
            }

            tbody.appendChild(row);
        });
    }
}