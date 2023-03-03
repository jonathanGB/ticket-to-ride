import React from 'react';
import ReactDOM from 'react-dom/client';
import './Styling/index.styles.tsx';
import reportWebVitals from './reportWebVitals';
import App from './Components/App';
import { BrowserRouter } from 'react-router-dom';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <BrowserRouter>
    <App /> {/* The various pages will be displayed by the `Main` component. */}
  </BrowserRouter>
);

// function poll() {
//   fetch(`${window.location.pathname}/state`).then(response => response.json()).then(game_state => {
//     // my logic.

//     setTimeout(poll, 1000);
//   })
// }

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
