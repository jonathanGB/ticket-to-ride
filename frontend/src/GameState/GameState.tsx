import { GamePhase } from "./GamePhase"
import { CardDealerState } from "./CardDealerState";
import { PlayerState } from "./PlayerState";
export class GameState {
    phase?: GamePhase;
    turn?: Number;
    card_dealer_state?: CardDealerState;
    players_state?: Array<PlayerState>;
}