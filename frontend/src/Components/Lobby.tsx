import React, { CSSProperties, FormEvent } from 'react';
import '../CSS/Lobby.css';
import PlayerCustom from './PlayerCustom';
import ColorPicker from './ColorPicker';
import PlayerColor from '../GameState/PlayerColor'
import TrainAnimation from './TrainAnimation';
import { GameState } from '../GameState/GameState';
import { PlayerState } from '../GameState/PlayerState';
import { click } from '@testing-library/user-event/dist/click';

interface PlayerNameFormCollection extends HTMLFormControlsCollection {
  playerName: HTMLInputElement,
}

interface PlayerNameFormElement extends HTMLFormElement {
  readonly elements: PlayerNameFormCollection,
}

class Lobby extends React.Component<{gameState: GameState}, {myName?: string}> {
  constructor(props: any) {
    super(props);
  }
  handleColorChange = (event: any) => {
    this.updatePlayer("color", event.target.value)
  }
  handleNameChange = (e: FormEvent<PlayerNameFormElement>) => {
    e.preventDefault();

    this.updatePlayer("name", e.currentTarget.elements.playerName.value);
  }
  private updatePlayer = async (changeType: string, value: string) => {
    let bodyRequest: any;
    let url: string = ""
    console.log("this before if/else: ", this)
    if (changeType == "name"){
      bodyRequest = {new_name: value};
      url = window.location + '/player/name'
    }
    else{
      bodyRequest = {new_color: value};
      url = window.location + '/player/color'
    }
    console.log("this before try: ", this)
    try {
      console.log("this before fetch: ", this)
      const response = await fetch(url, {
        method: 'PUT',
        body: JSON.stringify(bodyRequest),
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
      });
      console.log("this after fetch: ", this)
      let out = await response.json();
      console.log("out: ", out)
      if (!response.ok) {
        throw new Error(`Error! status: ${response.status}`);
      }
    } catch (error) {
      if (error instanceof Error) {
        console.log('error message: ', error.message);
        return error.message;
      } else {
        console.log('unexpected error: ', error);
        return 'An unexpected error occurred';
      }
    }
}
  render() {
    let namesList: any;
    let myColor: any;
    let selfPlayer: PlayerState = new PlayerState;
    if (this.props.gameState.players_state) {
      namesList = this.props.gameState.players_state.forEach( player => {
        if(player.private_player_state){
          selfPlayer = player;
        }
        else{
          return <PlayerCustom name = {player.public_player_state.name}  color = {player.public_player_state.color} isSelf = {false}/>
        }
      })
    }

    return <div className="Lobby-header">
      <div className='OtherPlayers-header'>
          {namesList}
      </div>
      <div className='SelfPlayer-header'>
          <PlayerCustom color = {selfPlayer.public_player_state.color} name = {selfPlayer.public_player_state.name} isSelf = {true}/>
      </div>
      <form onSubmit={this.handleNameChange}>
        <label>Enter name:</label>
        <input name= "playerName" type="text"/>
        <input type="submit" value="Submit"/>
      </form>
      <ColorPicker selfColor = {myColor} onChange = {this.handleColorChange}/>
      <TrainAnimation/>
    </div>
  }
};

export default Lobby;