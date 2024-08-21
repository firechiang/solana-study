import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import * as buffer from "buffer";
import reportWebVitals from './reportWebVitals';

window.Buffer = buffer.Buffer;

const root = ReactDOM.createRoot(
    document.getElementById('root') as HTMLElement
);

root.render(
    // 这是严格模式组件在开发环境下有些hook组件会默认执行2次来验证代码，如果不想让它执行2次，可以将 React.StrictMode 注释掉
    // <React.StrictMode>
    //     <App />
    // </React.StrictMode>
    <App />
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
