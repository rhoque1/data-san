import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

function App() {
  const [testResult, setTestResult] = useState<string>('Checking Tauri runtime...');
  const [drives, setDrives] = useState<any[]>([]);
  const [selectedDrive, setSelectedDrive] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [safetyResult, setSafetyResult] = useState<string>('');
  const [sanitizeResult, setSanitizeResult] = useState<string>('');
  const [systemSpecs, setSystemSpecs] = useState<any | null>(null);
  const [specsError, setSpecsError] = useState<string>("");
  const [displayResolution, setDisplayResolution] = useState<string>("");

  useEffect(() => {
    if (window.__TAURI__) {
      setTestResult('✅ Tauri runtime loaded successfully');
    } else {
      setTestResult('❌ ERROR: Not running in Tauri context - use "npm run tauri dev"');
    }
  }, []);

  useEffect(() => {
    setDisplayResolution(`${window.screen.width} x ${window.screen.height}`);
  }, []);

  const testConnection = async () => {
    if (!window.__TAURI__) {
      setTestResult('❌ Cannot test - not in Tauri context');
      return;
    }
    setIsLoading(true);
    setTestResult('🔄 Testing backend connection...');
    try {
      const result = await invoke<string>('test_system_info');
      setTestResult(`✅ Success: ${result}`);
    } catch (error) {
      setTestResult(`❌ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const detectDrives = async () => {
    if (!window.__TAURI__) {
      setTestResult('❌ Cannot detect drives - not in Tauri context');
      return;
    }
    setIsLoading(true);
    setTestResult('🔄 Detecting drives...');
    try {
      const driveList = await invoke<any[]>('detect_drives');
      setDrives(driveList);
      setTestResult(`✅ Drives detected: ${driveList.length} found`);
    } catch (error) {
      setTestResult(`❌ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const checkSafety = async () => {
    if (!window.__TAURI__ || !selectedDrive) {
      setSafetyResult('❌ Please select a drive and ensure Tauri context');
      return;
    }
    setIsLoading(true);
    setSafetyResult('🔄 Checking safety...');
    try {
      const result = await invoke<boolean>('check_safety', { driveLetter: selectedDrive });
      setSafetyResult(result ? '✅ Safe to proceed with confirmation' : '❌ Unsafe operation');
    } catch (error) {
      setSafetyResult(`❌ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const sanitizeDrive = async () => {
    if (!window.__TAURI__ || !selectedDrive) {
      setSanitizeResult('❌ Please select a drive and ensure Tauri context');
      return;
    }
    if (safetyResult !== '✅ Safe to proceed with confirmation') {
      setSanitizeResult('❌ Safety check required first');
      return;
    }
    setIsLoading(true);
    setSanitizeResult('🔄 Sanitizing...');
    try {
      const result = await invoke<string>('sanitize_drive', { driveLetter: selectedDrive, confirm: true });
      setSanitizeResult(`✅ ${result}`);
    } catch (error) {
      setSanitizeResult(`❌ ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const getSystemSpecs = async () => {
    setSpecsError("");
    setIsLoading(true);
    try {
      const specs = await invoke<any>('get_system_specs');
      setSystemSpecs(specs);
    } catch (error) {
      setSpecsError(`❌ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="container">
      <h1>Data Sanitizer Pro - MVP</h1>
      <div className="status-section">
        <h2>System Status</h2>
        <p className="status-text">{testResult}</p>
      </div>
      <div className="test-section">
        <button
          onClick={testConnection}
          disabled={isLoading || !window.__TAURI__}
          className="test-button"
        >
          {isLoading ? 'Testing...' : 'Test Backend Connection'}
        </button>
        <button
          onClick={detectDrives}
          disabled={isLoading || !window.__TAURI__}
          className="test-button"
          style={{ marginLeft: '10px' }}
        >
          {isLoading ? 'Detecting...' : 'Detect Drives'}
        </button>
        <button
          onClick={getSystemSpecs}
          disabled={isLoading || !window.__TAURI__}
          className="test-button"
          style={{ marginLeft: '10px' }}
        >
          {isLoading ? 'Getting Specs...' : 'Get System Specs'}
        </button>
      </div>
      {drives.length > 0 && (
        <div className="info-section">
          <h3>Detected Drives</h3>
          <ul>
            {drives.map((drive, index) => (
              <li key={index}>
                <input
                  type="radio"
                  name="drive"
                  value={drive.letter}
                  onChange={(e) => setSelectedDrive(e.target.value)}
                />{' '}
                {drive.letter} - {drive.label || 'No Label'} ({(drive.size / (1024 * 1024 * 1024)).toFixed(2)} GB)
                {drive.is_system ? ' (System)' : ''} - FS: {drive.file_system} - Serial: {drive.serial_number}
              </li>
            ))}
          </ul>
        </div>
      )}
      {selectedDrive && (
        <div className="test-section">
          <button
            onClick={checkSafety}
            disabled={isLoading || !window.__TAURI__}
            className="test-button"
          >
            {isLoading ? 'Checking...' : 'Check Safety'}
          </button>
          <p>{safetyResult}</p>
          <button
            onClick={sanitizeDrive}
            disabled={isLoading || !window.__TAURI__ || safetyResult !== '✅ Safe to proceed with confirmation'}
            className="test-button"
            style={{ marginLeft: '10px' }}
          >
            {isLoading ? 'Sanitizing...' : 'Sanitize Drive'}
          </button>
          <p>{sanitizeResult}</p>
        </div>
      )}
      {systemSpecs && (
        <div className="info-section">
          <h3>System Specs</h3>
          <ul>
            <li><b>OS Name:</b> {systemSpecs.os_name || 'N/A'}</li>
            <li><b>OS Version:</b> {systemSpecs.os_version || 'N/A'}</li>
            <li><b>Kernel Version:</b> {systemSpecs.kernel_version || 'N/A'}</li>
            <li><b>CPU Brand:</b> {systemSpecs.cpu_brand}</li>
            <li><b>CPU Cores:</b> {systemSpecs.cpu_cores}</li>
            <li><b>Total Memory:</b> {Math.round(systemSpecs.total_memory / 1024 / 1024)} GB</li>
            <li><b>Used Memory:</b> {Math.round(systemSpecs.used_memory / 1024 / 1024)} GB</li>
            {systemSpecs.gpu_name && (
              <li><b>GPU:</b> {systemSpecs.gpu_name}</li>
            )}
            <li><b>Display Resolution:</b> {displayResolution}</li>
            {systemSpecs.battery_percentage !== undefined && (
              <li><b>Battery Percentage:</b> {systemSpecs.battery_percentage.toFixed(1)}%</li>
            )}
            {systemSpecs.battery_cycle_count !== undefined && systemSpecs.battery_cycle_count !== null && (
              <li><b>Battery Cycle Count:</b> {systemSpecs.battery_cycle_count}</li>
            )}
          </ul>
        </div>
      )}
      {specsError && <div className="info-section"><p>{specsError}</p></div>}
      <div className="info-section">
        <h3>Development Notes</h3>
        <ul>
          <li>Use <code>npm run tauri dev</code></li>
          <li>Avoid <code>npm run dev</code></li>
        </ul>
      </div>
    </div>
  );
}

export default App;