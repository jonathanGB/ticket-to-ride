import { PlayerAction } from "./PlayerAction";
export class TurnActions {
  turn: Number;
  actions: Array<[PlayerAction]>;
  description: Array<String>;
  constructor() {
    this.turn = -1;
    this.actions = new Array();
    this.description = new Array();
  }
}
