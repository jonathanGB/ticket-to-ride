import React from "react";
import "../Styling/PlayerInfoBlock.styles.tsx";
import { PlayerState } from "../GameState/PlayerState";

class PlayerInfoBlock extends React.Component<{ selfPlayerState: PlayerState }> {
  constructor(props: any) {
    super(props);
  }

  render() {
    return <div className="PlayerInfo-header">
        <div className="Name-header">
          <h2>{this.props.selfPlayerState.public_player_state.name}</h2>
          <h2>{`${this.props.selfPlayerState.public_player_state.points}`}</h2>
        </div>
        {/* <div className="Destination-header">
          <label>Destinations</label>
          <label>{`${this.props.selfPlayerState.private_player_state?.selected_destination_cards[0].destination.CityStart.toString()}`}</label>
        </div> */}
        {/* <div className="Wagons-header">
          <label>{`${this.props.selfPlayerState.public_player_state.cars} wagons remaining`}</label>
        </div>
        <div className="TrainCard-header">
          <label>{`${this.props.selfPlayerState.private_player_state?.train_cards} wagons remaining`}</label>
        </div> */}
    </div>
  }
}
export default PlayerInfoBlock;