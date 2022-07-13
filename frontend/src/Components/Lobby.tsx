import React from 'react';
import '../CSS/Lobby.css';
import PlayerCustom from './PlayerCustom';
import NameForm from './NameForm';
import ColorPicker from './ColorPicker';

class Lobby extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    // let url = window.location + '/state';
    // fetch(url)
    // .then(res => res.json())
    // .then((out) => {
    //   let result: string = out.players;
    //   console.log(result)
    // })
    // .catch(err => { throw err });
    let playersJson = [{name: "Sumara"},{name: "Boubou"},{name: "Sofia"}]
    let namesList = playersJson.map(function(playerName){
      return <PlayerCustom name = {playerName.name}/>
    })
    return <div className="Lobby-header">
      <NameForm />
      <ColorPicker />
      {namesList}
    </div>
  }
};

export default Lobby;