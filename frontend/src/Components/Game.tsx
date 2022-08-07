import React from "react";
import "../CSS/App.css";
import Lobby from "./Lobby";
import { GameState } from "../GameState/GameState";
import { GamePhase } from "../GameState/GamePhase";

class Game extends React.Component<{}, { gameState: GameState }> {
  constructor(props: {} | Readonly<{}>) {
    super(props);
    this.state = { gameState: new GameState() };
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

  render() {
    //lobby has a different view than the other stages
    if (this.state.gameState.phase == GamePhase.InLobby) {
      return (
        <div className="Game-header">
          <Lobby gameState={this.state.gameState} />
        </div>
      );
    } else {
      console.log("Not in Lobby");
    }
  }
}

export default Game;
