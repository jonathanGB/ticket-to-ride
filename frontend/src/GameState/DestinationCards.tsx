import { CityToCity } from "./CityToCity";
export class DestinationCard {
  destination: CityToCity;
  points: Number;

  constructor(destination: CityToCity, points: Number) {
    this.destination = destination;
    this.points = points;
  }
}
