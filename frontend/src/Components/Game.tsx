import React from 'react';
import '../CSS/App.css';
import Lobby from './Lobby';

class Game extends React.Component<{}, { [phase: string]: string }> {
  constructor(props: {} | Readonly<{}>) {
    super(props);
    this.state = { phase: "" }
  }
  render() {
    let url = window.location + '/state';
    fetch(url)
    .then(res => res.json())
    .then((out) => {
      let result: string = out.phase;
      this.setState({phase: result});
    })
    .catch(err => { throw err });
    //lobby has a different view than the other stages
    if(this.state.phase == "in_lobby" ){
      return <div className="Game-header">
              <Lobby />
            </div>
    }
    else{
      console.log("hit here")
    }
    
  }
};

export default Game;