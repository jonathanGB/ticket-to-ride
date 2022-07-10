import React from 'react';
import '../CSS/App.css';
import Lobby from './Lobby';

class Game extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    let url = window.location + '/state';
    console.log("url: " + url)
    let phase = "";
    fetch(url)
    .then(res => res.json())
    .then((out) => {
      phase = out.phase;
    })
    .catch(err => { throw err });
    //lobby has a different view than the other stages
    if(phase == "in_lobby"){
      return <div className="Game-header">
              <Lobby />
            </div>
    }
    else{
      //for Game play, phases: starting, playing, last turn, and maybe done
    }
    
  }
};

export default Game;