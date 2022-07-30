import React, { CSSProperties } from 'react';
import '../CSS/Lobby.css';
import PlayerCustom from './PlayerCustom';
import ColorPicker from './ColorPicker';
import PlayerColor from '../PlayerColorEnum'
import TrainAnimation from './TrainAnimation';
import { Style } from 'util';

class Lobby extends React.Component<{}, {selfColor?: PlayerColor, defaultName?: string}> {
  constructor(props: {} | Readonly<{}>) {
    super(props);
    this.state = {selfColor: PlayerColor.Red, defaultName: 'Player1'}
    this.handleChange = this.handleChange.bind(this);
    this.handleNameChange = this.handleNameChange.bind(this);
  }
  handleChange(event: any) {
    this.setState({selfColor: event.target.value})
  }
  handleNameChange(event:any) {
    if(event.target.value.length < 10)
    {
      this.setState({defaultName: event.target.value});
    }
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
    let mystyle: CSSProperties = {
      backgroundColor: this.state.selfColor,
    }

    let inputElement = <input type="text" value={this.state.defaultName} onChange={this.handleNameChange} style = {mystyle}/>
    console.log(this.state.selfColor)
    return <div className="Lobby-header">
      <div className='OtherPlayers-header'>
          {namesList}
      </div>
      <div className='SelfPlayer-header'>
          <PlayerCustom input={inputElement} color = {this.state.selfColor} />
      </div>
      <ColorPicker selfColor = {this.state.selfColor} onChange = {this.handleChange}/>
      <TrainAnimation/>
    </div>
  }
};

export default Lobby;