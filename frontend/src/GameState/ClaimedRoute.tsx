import { City } from "./City";
import { CityToCity } from "./CityToCity";

export class ClaimedRoute {
    route: CityToCity;
    parallel_route_index: Number;
    length: Number;

    constructor(){
        this.route = {CityStart: City.Atlanta, CityEnd: City.Atlanta};
        this.parallel_route_index = -1
        this.length = -1
    }
}