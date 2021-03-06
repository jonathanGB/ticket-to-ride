import React from 'react';
import SvgComponent from './svg';
//testing testing 1 2 3
class Map extends React.Component {
    constructor(props: any) {
        super(props);
    }
    screenWidth = window.innerWidth;
    screenHeight = window.innerHeight;
    mystyle = {
        position: 'absolute',
        top: "0px", 
        left:"0px", 
        height:"100%",
        width:"100%",
        transform: "rotate(348deg)",
        textAlign: "center",
    };

    render() {
      return <div className="Map">
        
      <header className="Map-header">
        <SvgComponent style= {this.mystyle}/>
      </header>
    </div>
    }
};

export default Map;