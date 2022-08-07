import { PrivatePlayerState } from "./PrivatePlayerState";
import { PublicPlayerState } from "./PublicPlayerState";
export class PlayerState {
  public_player_state: PublicPlayerState;
  private_player_state?: PrivatePlayerState;

  constructor() {
    this.public_player_state = new PublicPlayerState();
    this.private_player_state = new PrivatePlayerState();
  }
}
