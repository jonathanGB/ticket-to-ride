import React from 'react';
import '../CSS/App.css';
import Map from './Map';

class Game extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    console.log("in game.tsx")
    return <div className="App-header">
      <h1>Hello game</h1>
      <header>
      <Map />
      </header>
    </div>
  }
};

export default Game;