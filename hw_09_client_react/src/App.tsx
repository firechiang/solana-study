import React from 'react';
import 'App.css';

function App() {
  // 取环境变量
  const test_str = process.env.REACT_APP_TEST_STR;
  return (
    <div className="App">
      <header className="App-header">
        <p>
          Edit <code>src/App.tsx</code> and save t
          { test_str }
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );
}

export default App;
