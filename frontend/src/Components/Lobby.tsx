import React, { useState, useEffect, ChangeEvent, FormEvent, Dispatch, SetStateAction } from 'react';
import * as Styles from './Lobby.styles';
import PlayerCustom from "./PlayerCustom";
import ColorPicker from "./ColorPicker";
import TrainAnimation from "./TrainAnimation";
import { GameState } from "../GameState/GameState";
import { PlayerState } from "../GameState/PlayerState";
import PlayerPlaceHolder from "./PlayerPlaceHolder";
import { TextField, Toggle } from "@fluentui/react";

interface LobbyProps {
  gameState: GameState;
  selfPlayerState: PlayerState;
}

const Lobby: React.FC<LobbyProps> = ({ gameState, selfPlayerState }) => {

  const [playerNameInput, setPlayerNameInput] = useState("");
  
  const handleIsReadyChange = (event: React.MouseEvent<HTMLElement>) => {
    updatePlayer({
        url: `${window.location.href}/player/is_ready`,
        request: {
            is_ready: !(event.currentTarget as HTMLInputElement).checked,
        },
    });
};

  const handleColorChange = (event: { target: { textContent: string; }; }) => {
    console.log(event);
    updatePlayer({
      url: `${window.location.href}/player/color`,
      request: {
        new_color: event.target.textContent.toLowerCase(),
      },
    });
  };

  const handleNameChange = (e: { preventDefault: () => void; }) => {
    e.preventDefault();
    updatePlayer({
      url: `${window.location.href}/player/name`,
      request: {
        new_name: playerNameInput,
      },
    });
  };

  const onChangePlayerNameInput = (event: FormEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    setPlayerNameInput(event.currentTarget.value);
  };  

  const updatePlayer = async (updatePlayerRequest: { url: any; request: any; }) => {
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

  const getOtherPlayers = () => {
     //todo: change to for loop
     let namesList: any = [];
     let selfPlayer: PlayerState = new PlayerState();
     if (gameState.players_state) {
       gameState.players_state.forEach((player: PlayerState) => {
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
     while (namesList.length < 4) {
       namesList.push(<PlayerPlaceHolder></PlayerPlaceHolder>);
     }
     return { namesList, selfPlayer };
  };

  let { namesList, selfPlayer } = getOtherPlayers();

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
              onChange={handleColorChange}
            />
            <form onSubmit={handleNameChange}>
              <TextField
                className={Styles.lobbyStyle.textFieldStyle}
                name="playerName"
                type="text"
                placeholder="Enter Player Name"
                onChange={onChangePlayerNameInput}
                value={playerNameInput}
              />
              <input type="submit" value="Submit" className="submitButton" />
            </form>
          </div>
          <Toggle
            label="Player Ready?"
            onChange={handleIsReadyChange}
            checked={selfPlayer.public_player_state.is_ready}
          />
        </div>
        <div>
          <div className={Styles.lobbyStyle.otherPlayersHeaderStyle}>
            {namesList}
          </div>
        </div>
      </div>
      <div className={Styles.lobbyStyle.trainContainerStyle}>
        <TrainAnimation />
      </div>
    </div>
  )
};

export default Lobby;
