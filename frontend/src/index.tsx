import React from 'react';
import ReactDOM from 'react-dom/client';
import './Styling/index.css';
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
 
reportWebVitals();
