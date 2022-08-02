import { TrainColor } from "./TrainColor";
import { DestinationCard } from "./DestinationCards";
export class PrivatePlayerState {
    train_cards: Map<TrainColor, Number>;
    pending_destination_cards: Array<DestinationCard>;
    selected_destination_cards: Array<DestinationCard>;

    constructor() {
        this.train_cards = new Map();
        this.pending_destination_cards = new Array;
        this.selected_destination_cards = new Array();
    }
}