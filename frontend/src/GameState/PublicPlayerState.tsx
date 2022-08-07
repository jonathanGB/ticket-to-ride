import PlayerColor from "./PlayerColor";
import { TurnActions } from "./TurnActions";
import { ClaimedRoute } from "./ClaimedRoute";
export class PublicPlayerState {
  id: Number;
  name: string;
  color: PlayerColor;
  is_ready: boolean;
  is_done_playing: boolean;
  cars: Number;
  points: Number;
  turn_actions: TurnActions;
  claimed_routes: Array<ClaimedRoute>;
  num_train_cards: Number;

  constructor() {
    this.id = -1;
    this.name = "";
    this.color = PlayerColor.Black;
    this.is_ready = false;
    this.is_done_playing = false;
    this.cars = 45;
    this.points = -1;
    this.turn_actions = new TurnActions();
    this.claimed_routes = new Array<ClaimedRoute>();
    this.num_train_cards = -1;
  }
}
