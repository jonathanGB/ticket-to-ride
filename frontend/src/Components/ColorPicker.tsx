import React from 'react';
import '../CSS/ColorPicker.css';

class ColorPicker extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
   return <div className='ColorPicker-header'>
        <p>Color</p>
        <div className="color" id='red'/>
        <div className="color" id='orange'/>
        <div className="color" id='yellow'/>
        <div className="color" id='green'/>
        <div className="color" id='blue'/>
        <div className="color" id='pink'/>
        <div className="color" id='white'/>
        <div className="color" id='black'/>
    </div>
  }
};

export default ColorPicker;