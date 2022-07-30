import React from 'react';
import '../CSS/ColorPicker.css';
import PlayerColor from '../PlayerColorEnum';

class ColorPicker extends React.Component<{selfColor?: PlayerColor, onChange: any}, {}> {
  constructor(props: any) {
    super(props);
  }
  
  render() {
   
   return <div className='ColorPicker-header'>
        <input className="color" id='red' onClick={this.props.onChange} value ={PlayerColor.Red}/>
        <input className="color" id='orange' onClick={this.props.onChange} value ={PlayerColor.Orange}/>
        <input className="color" id='yellow' onClick={this.props.onChange} value ={PlayerColor.Yellow}/>
        <input className="color" id='green' onClick={this.props.onChange} value ={PlayerColor.Green}/>
        <input className="color" id='blue' onClick={this.props.onChange} value ={PlayerColor.Blue}/>
        <input className="color" id='pink' onClick={this.props.onChange} value ={PlayerColor.Pink}/>
        <input className="color" id='white' onClick={this.props.onChange} value ={PlayerColor.White}/>
        <input className="color" id='black' onClick={this.props.onChange} value ={PlayerColor.Black}/>
    </div>
  }
};

export default ColorPicker;