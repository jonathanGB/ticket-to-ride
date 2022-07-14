import React from 'react';
import PlayerCustom from './PlayerCustom';
import '../CSS/ColorPicker.css'
import PlayerColor from '../PlayerColorEnum'

class NameForm extends React.Component<{selfColor?: PlayerColor}, { [value: string]: string }> {
    constructor(props: {} | Readonly<{}>) {
        super(props);
        this.state = {value: 'Player1'};
        
        this.handleChange = this.handleChange.bind(this);

      }
  
    handleChange(event:any) {
      if(event.target.value.length < 10)
      {
        this.setState({value: event.target.value});
      }
    }
  
    render() {
      return (
        <div className='NameForm-header'>
            <label>
                Enter Name:
                <input type="text" value={this.state.value} onChange={this.handleChange} />
            </label>
            <PlayerCustom name = {this.state.value} color = {this.props.selfColor}/>
        </div>
      );
    }
  };

export default NameForm