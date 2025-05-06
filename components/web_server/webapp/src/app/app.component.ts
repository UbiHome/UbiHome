import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { EventService } from '../services/events.service';
import { SubscriptionLike } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { NgFor } from '@angular/common';

@Component({
  selector: 'app-root',
  imports: [NgFor],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss'
})
export class AppComponent {
  private readonly eventSourceSubscription: SubscriptionLike;

  public events: String[] = [];

  constructor(
    private eventSourceService: EventService,
    private http: HttpClient,
) {
    this.http.get('/test').subscribe({
      complete: () => {
        console.log('Request completed');
      }
    })

    this.eventSourceSubscription = this.eventSourceService.connectToServerSentEvents()
        .subscribe({
                next: data => {
                  console.log('Event received:', data);
                    this.events.push(data.data);
                },
                error: error => {
                    //handle error
                }
            }
        );
}
  ngOnDestroy() {
    this.eventSourceSubscription.unsubscribe();
  }
}
