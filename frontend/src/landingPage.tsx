import React from 'react';
import ReactDOM from 'react-dom/client';
import './LandingPage.css';
import CreateGame from './Components/CreateGame';
import reportWebVitals from './reportWebVitals';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

function poll() {
  fetch(`${window.location.pathname}/state`).then(response => response.json()).then(game_state => {
    // my logic.

    setTimeout(poll, 1000);
  }).catch(error => {
    console.error(error);
    setTimeout(poll, 1000);
  }
  )
}

root.render(
  <React.StrictMode>
    <CreateGame />
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
