
function getColor(n) {
  if (n == 0) return "vermelho";
  if (n <= 3) return "ouro";
  if (n <= 6) return "prata";
  if (n <= 10) return "bronze";
  return "";
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

          <div className="run" style={{ position: "absolute", top: 10 }}>
            <div className="cell titulo">Placar</div>
            {allProblems.map(p => (<div className="cell problema">{p}</div>))}
          </div>


          {this.state.scoreOrder.map((dev) => {
            const team = this.state.teams[dev];
            var solved = 0;
            var penalty = 0;
            Object.values(team.problems).forEach(prob => {
              if (prob.solved) {
                solved++;
                penalty += prob.penalty;
              }
            });
            return <div className="run"
              key={dev}
              style={{
                  // zIndex: dev.id,
                  position: "absolute",
                  top: 10 + (1+this.state.scoreOrder.indexOf(dev)) * 90,
                  transition: "1s ease top",
                }}
                >
              <div className={"cell colocacao " + getColor(team.placement)}>{team.placement}</div>
              <div className="cell time">
                <div className="nomeEscola">{team.escola}</div>
                <div className="nomeTime">{team.name}</div>
              </div>
              <div className="cell problema"> 
                <div className="cima"> {solved}  </div>
                <div className="baixo"> {penalty} </div>
              </div>
              {allProblems.map((prob) => {
                if(prob in team.problems) {
                  const prob_v = team.problems[prob];
                  if(prob_v.solved) {
                    return <div className="cell problema verde"> 
                              <div className="cima"> +{prob_v.submissions}  </div>
                              <div className="baixo"> {prob_v.penalty} </div>
                           </div>;
                  }
                  return <div className={"cell problema "+ (prob_v.wait ? "amarelo" : "vermelho")}> 
                           <div className="cima">X </div>
                           <div className="baixo">({prob_v.submissions})</div> 
                         </div>;
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
