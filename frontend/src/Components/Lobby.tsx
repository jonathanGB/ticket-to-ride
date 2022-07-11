import React from 'react';
import '../CSS/Lobby.css';
import PlayerCustom from './PlayerCustom';

class Lobby extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="Lobby-header">
      <PlayerCustom />
      <PlayerCustom />
      <PlayerCustom />
      <PlayerCustom />
      <PlayerCustom />
    </div>
  }
};

export default Lobby;