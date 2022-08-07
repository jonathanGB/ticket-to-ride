<<<<<<< Updated upstream
import React, { ChangeEvent, CSSProperties, FormEvent } from "react";
import "../CSS/Lobby.css";
import PlayerCustom from "./PlayerCustom";
import ColorPicker from "./ColorPicker";
import PlayerColor from "../GameState/PlayerColor";
import TrainAnimation from "./TrainAnimation";
import { GameState } from "../GameState/GameState";
import { PlayerState } from "../GameState/PlayerState";
=======
import React from 'react';
import '../CSS/Lobby.css';
import PlayerCustom from './PlayerCustom';
>>>>>>> Stashed changes

interface PlayerNameFormCollection extends HTMLFormControlsCollection {
  playerName: HTMLInputElement;
}

interface PlayerNameFormElement extends HTMLFormElement {
  readonly elements: PlayerNameFormCollection;
}

interface URL {
  url: string;
}

interface SetPlayerReadyRequest extends URL {
  request: {
    is_ready: boolean;
  };
}

interface ChangePlayerNameRequest extends URL {
  request: {
    new_name: string;
  };
}

interface ChangePlayerColorRequest extends URL {
  request: {
    new_color: PlayerColor;
  };
}

type UpdatePlayerRequest =
  | SetPlayerReadyRequest
  | ChangePlayerNameRequest
  | ChangePlayerColorRequest;

class Lobby extends React.Component<
  { gameState: GameState },
  { myName?: string }
> {
  constructor(props: any) {
    super(props);
  }
  handleIsReadyChange = (event: ChangeEvent<HTMLInputElement>) => {
    console.log(event.target.checked);
    this.updatePlayer({
      url: `${window.location}/player/is_ready`,
      request: {
        is_ready: event.target.checked,
      },
    });
  };
  handleColorChange = (event: ChangeEvent<HTMLInputElement>) => {
    this.updatePlayer({
      url: `${window.location}/player/color`,
      request: {
        new_color: event.target.value as PlayerColor,
      },
    });
  };
  handleNameChange = (e: FormEvent<PlayerNameFormElement>) => {
    e.preventDefault();
    this.updatePlayer({
      url: `${window.location}/player/name`,
      request: {
        new_name: e.currentTarget.elements.playerName.value,
      },
    });
  };
  private updatePlayer = async (updatePlayerRequest: UpdatePlayerRequest) => {
    const { url, request } = updatePlayerRequest;

    console.log(request);
    try {
      const response = await fetch(url, {
        method: "PUT",
        body: JSON.stringify(request),
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
      });
      let out = await response.json();
      if (!out.success) {
        console.log("Error message from server: ", out.error_message);
        throw new Error(`Error! status: ${response.status}`);
      }
    } catch (error) {
      if (error instanceof Error) {
        console.log("error message: ", error.message);
        return error.message;
      } else {
        console.log("unexpected error: ", error);
        return "An unexpected error occurred";
      }
    }
  };

  render() {
    let namesList: any;
    let myColor: any;
    let selfPlayer: PlayerState = new PlayerState();
    if (this.props.gameState.players_state) {
      namesList = this.props.gameState.players_state.forEach((player) => {
        if (player.private_player_state) {
          selfPlayer = player;
        } else {
          return (
            <PlayerCustom
              name={player.public_player_state.name}
              color={player.public_player_state.color}
              isSelf={false}
            />
          );
        }
      });
    }

    return (
      <div className="Lobby-header">
        <div className="OtherPlayers-header">{namesList}</div>
        <div className="SelfPlayer-header">
          <PlayerCustom
            color={selfPlayer.public_player_state.color}
            name={selfPlayer.public_player_state.name}
            isSelf={true}
          />
        </div>
        <form onSubmit={this.handleNameChange}>
          <label>Enter name:</label>
          <input name="playerName" type="text" />
          <input type="submit" value="Submit" />
        </form>
        <label>Player Ready?</label>
        <input
          type="checkbox"
          onChange={this.handleIsReadyChange}
          checked={selfPlayer.public_player_state.is_ready}
        />
        <ColorPicker selfColor={myColor} onChange={this.handleColorChange} />
        <TrainAnimation />
      </div>
    );
  }
}

export default Lobby;
