import { TrainColor } from "./TrainColor";
export class CardDealerState {
    open_train_card_deck?: TrainColor[];
    close_train_card_deck_size: Number;
    discarded_train_card_deck_size: Number;
    destination_card_deck_size: Number;
    constructor () {
        this.close_train_card_deck_size = 0;
        this.discarded_train_card_deck_size = 0;
        this.destination_card_deck_size = 0;
    }
}