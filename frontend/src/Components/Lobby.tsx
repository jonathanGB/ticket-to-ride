import React, { ChangeEvent, CSSProperties, FormEvent, ReactNode } from "react";
import * as Styles from "./Lobby.styles";
import PlayerCustom from "./PlayerCustom";
import ColorPicker from "./ColorPicker";
import PlayerColor from "../GameState/PlayerColor";
import TrainAnimation from "./TrainAnimation";
import { GameState } from "../GameState/GameState";
import { PlayerState } from "../GameState/PlayerState";
import PlayerPlaceHolder from "./PlayerPlaceHolder";

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

class Lobby extends React.Component<{ gameState: GameState, selfPlayerState: any }
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

  getOtherPlayers = () => {
    //todo: change to for loop
    let namesList: any = [];
    let selfPlayer: PlayerState = new PlayerState();
    if (this.props.gameState.players_state) {
      this.props.gameState.players_state.forEach((player) => {
        if (player.private_player_state != null) {
          selfPlayer = player;
        } else {
          namesList.push(
            <PlayerCustom
              name={player.public_player_state.name}
              color={player.public_player_state.color}
              isSelf={false}
              isReady={player.public_player_state.is_ready}
            ></PlayerCustom>
          );
        }
      });
    }
    this.props.selfPlayerState(selfPlayer);
    while (namesList.length < 4) {
      namesList.push(<PlayerPlaceHolder></PlayerPlaceHolder>);
    }
    return { namesList, selfPlayer };
  };
  render() {
    let { namesList, selfPlayer } = this.getOtherPlayers();
    return (
      <div className={Styles.lobbyStyle.lobbyHeaderStyle}>
        <div className={Styles.lobbyStyle.playerSectionsStyle}>
          <div className={Styles.lobbyStyle.selfPlayerSectionStyle}>
            <div className={Styles.lobbyStyle.selfPlayerHeaderStyle}>
              <PlayerCustom
                color={selfPlayer.public_player_state.color}
                name={selfPlayer.public_player_state.name}
                isSelf={true}
                isReady={selfPlayer.public_player_state.is_ready}
              />
            </div>
            <div className={Styles.lobbyStyle.playerChoiceContainerStyle}>
              <ColorPicker
                selfColor={selfPlayer.public_player_state.color}
                onChange={this.handleColorChange}
              />
              <form onSubmit={this.handleNameChange}>
                <input
                  className={Styles.lobbyStyle.textFieldStyle}
                  name="playerName"
                  type="text"
                  placeholder="Enter Player Name"
                />
                <input type="submit" value="Submit"  className="submitButton"/>
              </form>
            </div>
            <label>
              Player Ready?
              <input
                type="checkbox"
                onChange={this.handleIsReadyChange}
                checked={selfPlayer.public_player_state.is_ready}
              />
            </label>
          </div>
          <div>
            <div className={Styles.lobbyStyle.otherPlayersHeaderStyle}>{namesList}</div>
          </div>
        </div>
        <div className={Styles.lobbyStyle.trainContainerStyle}>
          <TrainAnimation />
        </div>
      </div>
    );
  }
}

export default Lobby;
