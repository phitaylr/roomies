let uploadedFile = null;
let isProcessing = false;
let currentResult = null;
let currentEventName = null;

document.addEventListener('DOMContentLoaded', () => {
    console.log('Page loaded');
    console.log('window.__TAURI_INTERNALS__:', window.__TAURI_INTERNALS__);
    
    // File input handler
    document.getElementById('fileInput').addEventListener('change', (e) => {
        uploadedFile = e.target.files[0];
        console.log('File selected:', uploadedFile?.name);
        document.getElementById('solveBtn').disabled = !uploadedFile;
    });

    // Solve button handler
    document.getElementById('solveBtn').addEventListener('click', handleSolve);
    
    // Stop button handler
    document.getElementById('stopBtn').addEventListener('click', () => {
        alert('Stop functionality coming soon! The current search will complete.');
    });
    
    // Download button handler
    document.getElementById('downloadBtn').addEventListener('click', handleDownload);
    
    // Run Again button handler
    document.getElementById('runAgainBtn').addEventListener('click', handleRunAgain);
});

async function handleSolve() {
    if (!uploadedFile || isProcessing) return;
    isProcessing = true;

    const roomSize = parseInt(document.getElementById('roomSize').value);
    const iterations = parseInt(document.getElementById('iterations').value);
    const eventName = document.getElementById('eventName').value || 'Room Assignments';

    console.log('Processing with room size:', roomSize, 'iterations:', iterations);

    let progressUnlisten = null;
    let solutionUnlisten = null;

    try {
        const arrayBuffer = await uploadedFile.arrayBuffer();
        const bytes = Array.from(new Uint8Array(arrayBuffer));

        console.log('File bytes:', bytes.length);

        currentEventName = eventName;

        // Hide upload and config sections, show progress
        document.getElementById('upload-section').style.display = 'none';
        document.getElementById('config-section').style.display = 'none';
        document.getElementById('action-section').style.display = 'none';
        document.getElementById('progress').style.display = 'block';
        document.getElementById('results').style.display = 'none';
        document.getElementById('progressText').textContent = 'Starting...';
        document.getElementById('progressFill').style.width = '0%';

        // Listen for progress events
        try {
            const listen = window.__TAURI__?.event?.listen || 
                          window.__TAURI_INTERNALS__?.listen ||
                          window.__TAURI_IPC__?.listen;
            
            if (listen) {
                console.log('Setting up event listeners...');
                
                progressUnlisten = await listen('progress', (event) => {
                    console.log('Progress:', event.payload);
                    const progress = event.payload;
                    document.getElementById('progressFill').style.width = progress + '%';
                    document.getElementById('progressText').textContent = `Processing... ${progress}%`;
                });
                
                solutionUnlisten = await listen('solution_update', (event) => {
                    console.log('Solution update:', event.payload);
                    const update = event.payload;
                    document.getElementById('progressText').textContent = 
                        `Iteration ${update.iteration}: Score ${update.choice_score}, ` +
                        `${update.without_choices} without choices, imbalance ${update.imbalance}`;
                });
                
                console.log('Event listeners set up successfully');
            } else {
                console.log('No event listener API found, using indeterminate progress');
                document.getElementById('progressFill').classList.add('indeterminate');
            }
        } catch (e) {
            console.log('Could not set up listeners:', e);
            document.getElementById('progressFill').classList.add('indeterminate');
        }

        // Call Rust backend
        console.log('Calling Tauri invoke...');
        const result = await window.__TAURI_INTERNALS__.invoke('solve_rooms', {
            fileData: bytes,
            roomSize: roomSize,
            iterations: iterations
        });

        // Clean up listeners
        if (progressUnlisten) progressUnlisten();
        if (solutionUnlisten) solutionUnlisten();
        
        document.getElementById('progressFill').classList.remove('indeterminate');

        console.log('Got result:', result);
        currentResult = result;
        displayResults(result);
        
    } catch (error) {
        console.error('Error:', error);
        alert('Error: ' + error);
        
        // Show config sections again on error
        document.getElementById('upload-section').style.display = 'block';
        document.getElementById('config-section').style.display = 'block';
        document.getElementById('action-section').style.display = 'block';
        document.getElementById('progress').style.display = 'none';
        document.getElementById('progressFill').classList.remove('indeterminate');
        
        if (progressUnlisten) progressUnlisten();
        if (solutionUnlisten) solutionUnlisten();
    } finally {
        isProcessing = false;
    }
}

async function handleDownload() {
    if (!currentResult) {
        alert('No results to download');
        return;
    }
    
    try {
        document.getElementById('downloadBtn').disabled = true;
        document.getElementById('downloadBtn').textContent = 'Generating PDF...';
        
        const resultJson = JSON.stringify(currentResult);
        
        const filePath = await window.__TAURI_INTERNALS__.invoke('generate_pdf_report', {
            resultJson: resultJson,
            eventName: currentEventName || 'Room Assignments'
        });
        
        alert('PDF saved to: ' + filePath);
    } catch (error) {
        console.error('Error:', error);
        alert('Error: ' + error);
    } finally {
        document.getElementById('downloadBtn').disabled = false;
        document.getElementById('downloadBtn').textContent = 'Download Results';
    }
}

function displayResults(result) {
    document.getElementById('progress').style.display = 'none';
    document.getElementById('results').style.display = 'block';

    const summary = document.getElementById('resultsSummary');
    summary.innerHTML = `
        <div class="summary-item">
            <span class="summary-label">Choice Satisfaction Score</span>
            <span class="summary-value">${result.choice_score}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">Room Balance</span>
            <span class="summary-value">${result.imbalance}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">Without Choices</span>
            <span class="summary-value">${result.without_choices}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">Total Rooms</span>
            <span class="summary-value">${result.total_rooms}</span>
        </div>
    `;

    const detail = document.getElementById('resultsDetail');
    detail.innerHTML = Object.entries(result.rooms_by_category)
        .map(([category, rooms]) => `
            <div class="room-group">
                <div class="room-category">${category} Rooms</div>
                ${rooms.map((room, idx) => `
                    <div class="room">
                        <div class="room-title">Room ${idx + 1} - ${room.length} people</div>
                        <div class="room-members">
                            ${room.map(name => `<span class="member">${name}</span>`).join('')}
                        </div>
                    </div>
                `).join('')}
            </div>
        `).join('');
}


function handleRunAgain() {
    // Show upload and config sections again
    document.getElementById('upload-section').style.display = 'block';
    document.getElementById('config-section').style.display = 'block';
    document.getElementById('action-section').style.display = 'block';
    document.getElementById('results').style.display = 'none';
    document.getElementById('progress').style.display = 'none';
    
    // Scroll to top
    window.scrollTo({ top: 0, behavior: 'smooth' });
    
    // Increase iterations by 50%
    const currentIterations = parseInt(document.getElementById('iterations').value);
    const newIterations = Math.floor(currentIterations * 1.5);
    document.getElementById('iterations').value = newIterations;
    
    setTimeout(() => {
        alert(`Iterations increased to ${newIterations}. Adjust settings and click "Solve Room Assignments" to try again.`);
    }, 100);
}