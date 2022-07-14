import React from 'react';
import '../CSS/Lobby.css';
import PlayerCustom from './PlayerCustom';
import NameForm from './NameForm';
import ColorPicker from './ColorPicker';
import PlayerColor from '../PlayerColorEnum'

class Lobby extends React.Component<{}, {selfColor?: PlayerColor}> {
  constructor(props: {} | Readonly<{}>) {
    super(props);
    this.state = {selfColor: undefined}
    this.handleChange = this.handleChange.bind(this);
  }
  handleChange(event: any) {
    this.setState({selfColor: event.target.value})
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
    let playersJson = [{name: "Sumara", color: undefined},{name: "Boubou", color: undefined},{name: "Sofia", color: undefined}]
    let namesList = playersJson.map(({name, color}) => {
      return <PlayerCustom name = {name}  color = {color}/>
    })
    console.log(this.state.selfColor)
    return <div className="Lobby-header">
      <NameForm selfColor = {this.state.selfColor} />
      <ColorPicker selfColor = {this.state.selfColor} onChange = {this.handleChange}/>
      {namesList}
    </div>
  }
};

export default Lobby;