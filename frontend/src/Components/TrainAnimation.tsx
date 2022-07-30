import React from 'react';
import '../CSS/TrainAnimation.css';

class TrainAnimation extends React.Component<{}, {}> {
  constructor(props: any) {
    super(props);
  }

  render() {

    return <div className='train'>
             <div className='background'/>
        </div>
  }
};

export default TrainAnimation;