import React from 'react';
import '../CSS/App.css';
import Main from './Main';

class App extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    console.log("hit app")
    return <div className="main-header">
      <Main />
    </div>
  }
};

export default App;
