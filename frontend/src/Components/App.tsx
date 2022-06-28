import React from 'react';
import '../CSS/App.css';
import Map from './Map';

class App extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="App-header">
      <header>
        <form action="/create" method="post">
          <input className="create-button" type="submit" value="Create game" />
        </form>
      </header>
      <Map />
    </div>
  }
};

export default App;
