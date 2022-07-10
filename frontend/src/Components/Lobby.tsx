import React from 'react';
import '../CSS/App.css';

class Lobby extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="Lobby-header">
      <h1>Hello game</h1>
    </div>
  }
};

export default Lobby;