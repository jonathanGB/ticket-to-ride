import React from 'react';
import PlayerCustom from './PlayerCustom';
import '../CSS/ColorPicker.css'

class NameForm extends React.Component<{}, { [value: string]: string }> {
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
            <PlayerCustom name = {this.state.value}/>
        </div>
      );
    }
  };

export default NameForm