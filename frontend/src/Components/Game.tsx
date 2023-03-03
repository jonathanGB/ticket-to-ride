import React from "react";
import "../Styling/App.css";
import * as Styles from "./Game.styles";
import Lobby from "./Lobby";
import { GameState } from "../GameState/GameState";
import { GamePhase } from "../GameState/GamePhase";
import Starting from "./Starting";
import { PlayerState } from "../GameState/PlayerState";

class Game extends React.Component<{}, { gameState: GameState, selfPlayerState: PlayerState }> {
  constructor(props: any) {
    super(props);
    this.state = { gameState: new GameState(), selfPlayerState: new PlayerState() };
    this.getGameState();
  }
  private async getGameState() {
    let url = window.location + "/state";
    try {
      let response = await fetch(url);
      let out = await response.json();
      this.setState({ gameState: out });
    } catch (err) {
      console.error(err);
    }
    setTimeout(this.getGameState.bind(this), 500);
  }
  changePlayerState = (update: PlayerState) => {
    this.setState({selfPlayerState: update});
  }

  render() {
    //lobby has a different view than the other stages
    if (this.state.gameState.phase == GamePhase.InLobby) {
      return (
        <div className={Styles.mainStyle.gameHeaderStyle}>
          <Lobby gameState={this.state.gameState} selfPlayerState={this.changePlayerState} />
        </div>
      );
    } else {
      return (
        <div className={Styles.mainStyle.startingStyle}>
          <Starting gameState={this.state.gameState} selfPlayerState={this.state.selfPlayerState} />
        </div>
      );
    }
  }
}

export default Game;
