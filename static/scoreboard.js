
function getColor(n) {
  if (n == 0) return 'red';
  if (n <= 3) return 'yellow';
  if (n <= 6) return 'gray';
  if (n <= 10) return 'orange';
  return 'black';
}

function getAnswer(t) {
  if (t == "Yes") {
    return <img src="assets/yes.png" />
  }
  else if (t == "No") {
    return <img src="assets/no.png" />
  }
  else if (t == "Wait") {
    return <img src="assets/question.png" />
  }
  return "Error";
}

class Scoreboard extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      scoreOrder: [],
      teams: []
    };
  }

  updateComponent() {
    fetch("http://localhost:3030/score")
    .then(res => res.json())
    .then(
      (result) => {
        console.log("Obtained results from api:", result)
        this.setState({scoreOrder: result[0], teams: result[1] });
      },
      // Nota: É importante lidar com os erros aqui
      // em vez de um bloco catch() para não recebermos
      // exceções de erros dos componentes.
      (error) => {
        console.log("ajax request error:", error);
        // this.setState({
        //   isLoaded: true,
        //   error
        // });
      }
    )
  }

  componentDidMount() {
    console.log("component did mount!");
    setInterval(() => {
      this.updateComponent();
    }, 1000);
  }

  /*
  escola: "UFPA"
​​​
login: "teambrbr01"
​​​
name: "[UFPA] Greedy d+"
​​​
placement: 240
​​​
problems: {…}
​​​​
E: Object { solved: true, submissions: 1, penalty: 48 }
​​​​
I: Object { solved: true, submissions: 1, penalty: 108 }
​​​​
K: Object { solved: false, submissions: 0, penalty: 0 }
​​​​
M: {…}*/
  render() {
    const allProblems = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M"];
    return (
      <>
        <div border="1">
          {this.state.scoreOrder.map((dev) => {
            const team = this.state.teams[dev];
            return <div className="run"
              key={dev}
              style={{
                  // zIndex: dev.id,
                  position: "absolute",
                  top: 10 + this.state.scoreOrder.indexOf(dev) * 90,
                  transition: "1s ease top",
                }}
                >
              <div className="cell colocacao"
              style={{
                backgroundColor: getColor(team.placement)
              }}> {team.placement} </div>
              <div className="cell time">
                <div className="nomeEscola">{team.escola}</div>
                <div className="nomeTime">{team.name}</div>
              </div>
              {allProblems.map((prob) => {
                if(prob in team.problems) {
                  return <div className="cell problema"> {prob} </div>;
                }
                else  {
                  return <div className="cell problema"> - </div>;
                }
              })}
            </div>
          })}
        </div>
      </>
    );
  }
}
  
ReactDOM.render(
  <Scoreboard />,
  document.getElementById('root')
);
