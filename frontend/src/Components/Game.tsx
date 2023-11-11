import React, { useState, useEffect } from 'react';
import * as Styles from './Game.styles';
import Lobby from './Lobby';
import { GameState } from '../GameState/GameState';
import { GamePhase } from '../GameState/GamePhase';
import Starting from './Starting';
import { PlayerState } from '../GameState/PlayerState';

const Game = () => {
  const [gameState, setGameState] = useState(new GameState());
  const [selfPlayerState, setSelfPlayerState] = useState(new PlayerState());

  const getGameState = async () => {
    let url = window.location + '/state';
    try {
      let response = await fetch(url);
      let out = await response.json();
      setGameState(out);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    getGameState();
    const intervalId = setInterval(getGameState, 500);
    return () => clearInterval(intervalId); // Cleanup on unmount
  }, []); // Empty dependency array to run only on mount


  // Lobby has a different view than the other stages
  if (gameState.phase === GamePhase.InLobby) {
    return (
      <div className={Styles.mainStyle.gameHeaderStyle}>
        <Lobby gameState={gameState} selfPlayerState={selfPlayerState} />
      </div>
    );
  } else {
    return (
      <div className={Styles.mainStyle.startingStyle}>
        <Starting gameState={gameState} selfPlayerState={selfPlayerState} />
      </div>
    );
  }
};

export default Game;
