import React from "react";
import "../Styling/Starting.styles.tsx";
import { GameState } from "../GameState/GameState";
import Map from "./Map";
import PlayerInfoBlock from "./PlayerInfoBlock";
import { PlayerState } from "../GameState/PlayerState";

class Starting extends React.Component<{ gameState: GameState, selfPlayerState: PlayerState }> {
  constructor(props: any) {
    super(props);
  }

  render() {
    return <div className="Starting-header">
        <PlayerInfoBlock selfPlayerState={this.props.selfPlayerState}/>
        <Map/>
    </div>
  }
}
export default Starting;
